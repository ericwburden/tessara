# Tessara Roadmap Backlog With Acceptance Criteria

Derived from the updated roadmap dated April 17, 2026.

## Conventions

- **Completed sprints are locked.** Their backlog items below are regression, cleanup, or guardrail work only; they should not reopen net-new feature scope.
- **Open sprints are implementation backlogs.** The tickets are sized so one Codex task can usually take one ticket or a narrow slice of one ticket.
- **Ticket IDs** use the sprint prefix so they can be turned into GitHub issues directly.


- **Labels** are suggested GitHub issue labels and follow a consistent taxonomy: `phase:*`, `sprint:*`, `type:*`, `status:*`, and one to three `area:*` labels.
- **Depends on** uses ticket IDs from this document so issue dependencies can be translated directly into linked GitHub issues.
- **Milestone** uses the sprint name as the recommended GitHub milestone for that ticket.
## Common delivery gates

Every ticket inherits these baseline acceptance gates when applicable:

- All touched routes render through native Leptos SSR ownership, not `application.rs`, `inner_html` injection, `/bridge/*`, or retained bridge assets.
- No new user-facing behavior lands in legacy bridge code unless the ticket explicitly deletes that shim in the same sprint.
- Any touched backend slice in `auth`, `hierarchy`, `forms`, `workflows`, `submissions`, or `reporting` moves toward `router`, `handlers`, `service`, `repo`, and `dto` boundaries.
- Client-visible errors use stable application codes/messages and do not expose raw database or internal error strings.
- `local-launch.ps1` and `uat-sprint.ps1` pass, and touched routes are clean for route ownership, hydration, and browser-console output.

---

# Frontend Platform Foundation

## Platform Sprint A: Cargo-Leptos Foundation

### PSA-01: Establish `cargo-leptos` workspace metadata
**Scope:** Build / workspace / platform  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:feature`, `status:backlog`, `area:platform`
**Depends on:** None
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Establish `cargo-leptos` workspace metadata** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to add or finalize multi-package `cargo-leptos` workspace metadata with `tessara-api` as the server binary and `tessara-web` as the frontend library. It also includes work to make local dev and CI builds use the same platform entry points. This ticket spans **Build / workspace / platform** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add or finalize multi-package `cargo-leptos` workspace metadata with `tessara-api` as the server binary and `tessara-web` as the frontend library.
- Make local dev and CI builds use the same platform entry points.

**Acceptance criteria:**
- `cargo leptos build` completes without manual patching.
- The server binary still starts as the single deployable backend process.
- Build instructions in the repo are updated so a new contributor can reproduce the build from clean checkout.

### PSA-02: Serve built frontend assets from the existing `axum` binary
**Scope:** Backend / frontend integration  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:platform`
**Depends on:** `PSA-01`
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Serve built frontend assets from the existing `axum` binary** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to wire the compiled wasm/js/css output into the existing server process. It also includes work to ensure cache-busted asset references are emitted from the SSR shell. This ticket spans **Backend / frontend integration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Wire the compiled wasm/js/css output into the existing server process.
- Ensure cache-busted asset references are emitted from the SSR shell.

**Acceptance criteria:**
- The browser receives compiled frontend assets from the existing server binary.
- Asset URLs change when asset contents change.
- A hard refresh after rebuild loads the current assets without manual cache clearing.

### PSA-03: Hydrated router parity for preserved URLs
**Scope:** Frontend routing / SSR  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:feature`, `status:backlog`, `area:frontend`, `area:platform`
**Depends on:** `PSA-01`, `PSA-02`
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Hydrated router parity for preserved URLs** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to put the preserved route surface behind a hydrated Leptos router. It also includes work to keep current URLs stable while the runtime contract changes. This ticket spans **Frontend routing / SSR** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Put the preserved route surface behind a hydrated Leptos router.
- Keep current URLs stable while the runtime contract changes.

**Acceptance criteria:**
- Direct navigation to existing preserved URLs renders the correct page.
- In-app navigation between preserved URLs works without breaking hydration.
- SSR markup and hydrated behavior match on first load.

### PSA-04: Move transitional bridge code out of Rust string literals
**Scope:** Frontend assets / cleanup  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:refactor`, `status:backlog`, `area:frontend`, `area:platform`
**Depends on:** `PSA-01`
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Move transitional bridge code out of Rust string literals** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to extract retained bridge scripts from Rust HTML-string shells into explicit frontend assets. It also includes work to remove any new inline script growth from Rust-rendered page bodies. This ticket spans **Frontend assets / cleanup** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Extract retained bridge scripts from Rust HTML-string shells into explicit frontend assets.
- Remove any new inline script growth from Rust-rendered page bodies.

**Acceptance criteria:**
- Retained bridge scripts are loaded from named asset files rather than large Rust string literals.
- No new bridge logic is added inside Rust HTML templates.
- The app still functions on preserved routes after the extraction.

### PSA-05: Shared stylesheet through the `cargo-leptos` pipeline
**Scope:** Frontend styling / build pipeline  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:feature`, `status:backlog`, `area:frontend`, `area:platform`
**Depends on:** `PSA-01`
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Shared stylesheet through the `cargo-leptos` pipeline** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to route the shared stylesheet through the same asset pipeline as the rest of the frontend. It also includes work to eliminate parallel stylesheet delivery paths where practical. This ticket spans **Frontend styling / build pipeline** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Route the shared stylesheet through the same asset pipeline as the rest of the frontend.
- Eliminate parallel stylesheet delivery paths where practical.

**Acceptance criteria:**
- The shared stylesheet is emitted by the platform pipeline.
- Styling remains stable across the preserved route surface after the move.
- There is a single documented path for updating shared app styles.

### PSA-06: Platform smoke coverage
**Scope:** QA / CI  
**Labels:** `phase:platform`, `sprint:platform-a`, `type:test`, `status:backlog`, `area:platform`, `area:ci`
**Depends on:** `PSA-01`, `PSA-02`, `PSA-03`, `PSA-05`
**Milestone:** `Platform Sprint A - Cargo-Leptos Foundation`


**Description:**
This issue implements **Platform smoke coverage** in **Platform Sprint A - Cargo-Leptos Foundation**. The primary goal is to add platform-level smoke coverage that proves the server and frontend build/run contract. It also includes work to include SSR + hydration checks for at least one preserved route. This ticket spans **QA / CI** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add platform-level smoke coverage that proves the server and frontend build/run contract.
- Include SSR + hydration checks for at least one preserved route.

**Acceptance criteria:**
- CI or local smoke automation fails when the platform build is broken.
- At least one preserved route is verified for SSR render plus successful hydration.
- The smoke output is documented so sprint-close verification is repeatable.

---

## Platform Sprint B: Route Parity With Isolated Bridge

### PSB-01: Create a route ownership inventory
**Scope:** Frontend architecture / documentation  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:docs`, `status:backlog`, `area:frontend`, `area:documentation`, `area:platform`
**Depends on:** `PSA-06`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Create a route ownership inventory** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to inventory every preserved URL and assign it one of: native Leptos-owned, bridged, or legacy-only. It also includes work to record the intended replacement target for every bridged surface. This ticket spans **Frontend architecture / documentation** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Inventory every preserved URL and assign it one of: native Leptos-owned, bridged, or legacy-only.
- Record the intended replacement target for every bridged surface.

**Acceptance criteria:**
- There is a checked-in route inventory covering all preserved URLs.
- Every bridged route has a named native replacement target.
- The inventory is easy to compare against UAT output during sprint close.

### PSB-02: Move body-level metadata ownership into Leptos
**Scope:** Frontend shell / routing  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:feature`, `status:backlog`, `area:frontend`, `area:platform`
**Depends on:** `PSB-01`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Move body-level metadata ownership into Leptos** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to move route body metadata, page identity, and shell/runtime decisions out of ad hoc bridge paths and into Leptos-owned route components. This ticket spans **Frontend shell / routing** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Move route body metadata, page identity, and shell/runtime decisions out of ad hoc bridge paths and into Leptos-owned route components.

**Acceptance criteria:**
- Touched routes derive page/body metadata from Leptos-owned route code.
- Navigation state and page identity remain correct after reload and client-side navigation.
- No new metadata decisions are hidden in bridge-only code.

### PSB-03: Add feature-local transport boundaries
**Scope:** Frontend architecture / API client  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:refactor`, `status:backlog`, `area:frontend`, `area:backend`, `area:platform`
**Depends on:** `PSA-06`, `PSB-01`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Add feature-local transport boundaries** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to introduce per-feature transport/client boundaries rather than one bridge-global request layer. It also includes work to start with shared or high-churn surfaces. This ticket spans **Frontend architecture / API client** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Introduce per-feature transport/client boundaries rather than one bridge-global request layer.
- Start with shared or high-churn surfaces.

**Acceptance criteria:**
- At least two feature areas call the API through local transport modules rather than a bridge-global helper.
- Transport boundaries are typed enough that request/response drift is visible at compile time or narrow integration tests.
- New route work uses the feature-local pattern by default.

### PSB-04: Achieve parity for preserved routes under the Leptos runtime contract
**Scope:** Full stack  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:feature`, `status:backlog`, `area:runtime`, `area:platform`
**Depends on:** `PSB-01`, `PSB-02`, `PSB-03`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Achieve parity for preserved routes under the Leptos runtime contract** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to migrate preserved route bodies so the shell/runtime contract is Leptos-owned even if some bodies still use a retained bridge. This ticket spans **Full stack** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Migrate preserved route bodies so the shell/runtime contract is Leptos-owned even if some bodies still use a retained bridge.

**Acceptance criteria:**
- Preserved URLs continue to work without URL churn.
- Reload, deep-link, and in-app navigation work through the Leptos runtime contract.
- Any remaining bridge usage is isolated and explicit rather than the default architecture.

### PSB-05: Publish an explicit bridge inventory and burn-down list
**Scope:** Documentation / planning  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:docs`, `status:backlog`, `area:platform`, `area:documentation`
**Depends on:** `PSB-01`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Publish an explicit bridge inventory and burn-down list** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to create a living inventory of retained bridge assets, routes, and state owners. It also includes work to note the deletion condition for each one. This ticket spans **Documentation / planning** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Create a living inventory of retained bridge assets, routes, and state owners.
- Note the deletion condition for each one.

**Acceptance criteria:**
- Every remaining bridge asset or route has an owner and deletion target.
- The burn-down list distinguishes product, admin, and migration/operator surfaces.
- Sprint planning can point to specific bridge deletions rather than general clean-up goals.

### PSB-06: Route parity regression suite
**Scope:** QA / end-to-end  
**Labels:** `phase:platform`, `sprint:platform-b`, `type:test`, `status:backlog`, `area:platform`
**Depends on:** `PSB-04`, `PSB-05`
**Milestone:** `Platform Sprint B - Route Parity With Isolated Bridge`


**Description:**
This issue implements **Route parity regression suite** in **Platform Sprint B - Route Parity With Isolated Bridge**. The primary goal is to add regression coverage for deep-link, reload, auth redirect, and back/forward behavior across preserved routes. This ticket spans **QA / end-to-end** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add regression coverage for deep-link, reload, auth redirect, and back/forward behavior across preserved routes.

**Acceptance criteria:**
- At least one regression test covers direct entry, reload, and in-app navigation across preserved routes.
- Failures identify which route lost parity.
- The test suite can run in local sprint-close verification.

---

## Platform Sprint C: Split Heavy Routes And Start Bridge Removal

### PSC-01: Route-level code splitting for heavy operator routes
**Scope:** Frontend performance / architecture  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:refactor`, `status:backlog`, `area:frontend`, `area:operators`, `area:performance`
**Depends on:** `PSB-06`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Route-level code splitting for heavy operator routes** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to introduce route-level code splitting beginning with `/app/migration`. It also includes work to keep the shared shell light. This ticket spans **Frontend performance / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Introduce route-level code splitting beginning with `/app/migration`.
- Keep the shared shell light.

**Acceptance criteria:**
- `/app/migration` loads its heavy client code only when entered.
- Shared-shell payload size is reduced or held stable after the split.
- Direct navigation to the heavy route still hydrates successfully.

### PSC-02: Bundle-loading verification
**Scope:** QA / performance  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:feature`, `status:backlog`, `area:performance`, `area:platform`
**Depends on:** `PSC-01`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Bundle-loading verification** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to add test coverage that proves lazy bundles are requested only when their routes are entered. This ticket spans **QA / performance** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add test coverage that proves lazy bundles are requested only when their routes are entered.

**Acceptance criteria:**
- Automated coverage confirms route-specific bundles are not loaded on unrelated routes.
- Test failures show which heavy route regressed.
- The verification runs in sprint-close checks.

### PSC-03: Remove the bridge from the first preserved product route
**Scope:** Full stack / migration  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:feature`, `status:backlog`, `area:migration`, `area:platform`
**Depends on:** `PSB-06`, `PSC-01`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Remove the bridge from the first preserved product route** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to pick one preserved product route and complete its native replacement. It also includes work to delete the bridge-owned state or rendering path for that route. This ticket spans **Full stack / migration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Pick one preserved product route and complete its native replacement.
- Delete the bridge-owned state or rendering path for that route.

**Acceptance criteria:**
- The chosen product route works end to end without any bridge dependency.
- Related route tests pass on SSR render, hydration, refresh, and navigation.
- The bridge inventory marks the route as deleted rather than merely hidden.

### PSC-04: Remove the bridge from the first preserved internal/operator route
**Scope:** Full stack / migration  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:feature`, `status:backlog`, `area:migration`, `area:platform`, `area:operators`
**Depends on:** `PSB-06`, `PSC-01`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Remove the bridge from the first preserved internal/operator route** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to repeat the previous ticket for one internal/operator surface, ideally one heavy enough to validate the pattern. This ticket spans **Full stack / migration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Repeat the previous ticket for one internal/operator surface, ideally one heavy enough to validate the pattern.

**Acceptance criteria:**
- The chosen internal/operator route no longer requires the bridge.
- Operator behavior remains testable through the intended SSR surface.
- The route’s bridge-owned state path is removed from the codebase.

### PSC-05: Enforce console-clean and hydration-clean routes
**Scope:** QA / platform quality  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:feature`, `status:backlog`, `area:frontend`, `area:platform`
**Depends on:** `PSC-03`, `PSC-04`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Enforce console-clean and hydration-clean routes** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to make browser-console errors and hydration mismatches fail route-level checks for touched routes. This ticket spans **QA / platform quality** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Make browser-console errors and hydration mismatches fail route-level checks for touched routes.

**Acceptance criteria:**
- Touched route tests fail on hydration warnings or browser-console errors.
- Sprint-close checks report which route emitted the failure.
- The rule is documented so contributors know the failure mode is intentional.

### PSC-06: Shared-shell performance guard
**Scope:** Performance / CI  
**Labels:** `phase:platform`, `sprint:platform-c`, `type:ops`, `status:backlog`, `area:frontend`, `area:performance`, `area:ci`
**Depends on:** `PSC-01`
**Milestone:** `Platform Sprint C - Split Heavy Routes And Start Bridge Removal`


**Description:**
This issue implements **Shared-shell performance guard** in **Platform Sprint C - Split Heavy Routes And Start Bridge Removal**. The primary goal is to add a lightweight guardrail for shell payload, bundle count, or route-entry latency. This ticket spans **Performance / CI** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add a lightweight guardrail for shell payload, bundle count, or route-entry latency.

**Acceptance criteria:**
- There is at least one measurable shell-level performance budget or baseline.
- CI or local verification reports when the budget regresses.
- The budget excludes intentionally lazy-loaded heavy routes.

---

# Phase 1: Identity, Access, Organization, And Form Authoring

## Sprint 1A: User Management And Authentication (Complete — regression backlog only)

### 1A-R1: User CRUD regression coverage
**Scope:** QA / regression  
**Labels:** `phase:1-identity-access-forms`, `sprint:1A`, `type:test`, `status:regression-only`, `area:admin`
**Depends on:** None
**Milestone:** `Sprint 1A - User Management And Authentication`


**Description:**
This issue implements **1A-R1: User CRUD regression coverage** in **Sprint 1A - User Management And Authentication** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Automated coverage proves a tester can list, create, edit, and inspect users through the application UI.
- Regressions identify whether the failure is list, detail, create, or edit behavior.
- No direct DB setup is required beyond the normal local seed path.

### 1A-R2: Login failure and account-status regression coverage
**Scope:** QA / auth  
**Labels:** `phase:1-identity-access-forms`, `sprint:1A`, `type:test`, `status:regression-only`, `area:auth`, `area:admin`
**Depends on:** None
**Milestone:** `Sprint 1A - User Management And Authentication`


**Description:**
This issue implements **1A-R2: Login failure and account-status regression coverage** in **Sprint 1A - User Management And Authentication** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Failed login cases surface explicit end-user error states.
- Disabled or inactive accounts are denied access consistently.
- Current-user visibility remains correct after login and refresh.

### 1A-R3: Post-login home-entry contract test
**Scope:** QA / routing  
**Labels:** `phase:1-identity-access-forms`, `sprint:1A`, `type:test`, `status:regression-only`, `area:auth`, `area:admin`
**Depends on:** None
**Milestone:** `Sprint 1A - User Management And Authentication`


**Description:**
This issue implements **1A-R3: Post-login home-entry contract test** in **Sprint 1A - User Management And Authentication** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Post-login routing lands on the intended home/shell entry point.
- Refresh after login preserves the expected authenticated shell.
- Logout clears the authenticated entry state for the browser session.

---

## Sprint 1B: RBAC And Scoped Role Assignment (Complete — regression backlog only)

### 1B-R1: Role and capability-bundle regression suite
**Scope:** QA / admin  
**Labels:** `phase:1-identity-access-forms`, `sprint:1B`, `type:test`, `status:regression-only`, `area:authorization`, `area:admin`
**Depends on:** None
**Milestone:** `Sprint 1B - RBAC And Scoped Role Assignment`


**Description:**
This issue implements **1B-R1: Role and capability-bundle regression suite** in **Sprint 1B - RBAC And Scoped Role Assignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can create, edit, and inspect roles and capability bundles through the UI.
- Capability-bundle grid or table views remain readable at larger data volumes.
- Regressions distinguish role CRUD from capability-bundle display failures.

### 1B-R2: Scoped assignment and descendant-scope regression suite
**Scope:** QA / authorization  
**Labels:** `phase:1-identity-access-forms`, `sprint:1B`, `type:test`, `status:regression-only`, `area:authorization`
**Depends on:** None
**Milestone:** `Sprint 1B - RBAC And Scoped Role Assignment`


**Description:**
This issue implements **1B-R2: Scoped assignment and descendant-scope regression suite** in **Sprint 1B - RBAC And Scoped Role Assignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Assigned scope and descendant-scope behavior match the expected inheritance rules.
- UI assignment flows do not require manual IDs.
- Failing tests identify whether the issue is assignment, inheritance, or display logic.

### 1B-R3: Navigation and action-gating matrix coverage
**Scope:** QA / product behavior  
**Labels:** `phase:1-identity-access-forms`, `sprint:1B`, `type:test`, `status:regression-only`, `area:authorization`
**Depends on:** None
**Milestone:** `Sprint 1B - RBAC And Scoped Role Assignment`


**Description:**
This issue implements **1B-R3: Navigation and action-gating matrix coverage** in **Sprint 1B - RBAC And Scoped Role Assignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- At least one test matrix covers navigation and action gating for multiple role/scope combinations.
- Touched product surfaces do not expose unauthorized actions.
- Unauthorized actions fail safely and predictably if forced by URL or request.

---

## Sprint 1C: Organization Management (Complete — regression backlog only)

### 1C-R1: Hierarchy navigator regression coverage
**Scope:** QA / organization  
**Labels:** `phase:1-identity-access-forms`, `sprint:1C`, `type:test`, `status:regression-only`, `area:organization`
**Depends on:** None
**Milestone:** `Sprint 1C - Organization Management`


**Description:**
This issue implements **1C-R1: Hierarchy navigator regression coverage** in **Sprint 1C - Organization Management** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- The hierarchy tree navigator remains the primary browse affordance.
- Scoped users can browse their assigned subtree without flat-card fallbacks.
- Regressions identify whether the issue is tree load, selection, or detail sync.

### 1C-R2: Scope-aware naming regression coverage
**Scope:** QA / UI semantics  
**Labels:** `phase:1-identity-access-forms`, `sprint:1C`, `type:test`, `status:regression-only`, `area:frontend`, `area:authorization`, `area:organization`
**Depends on:** None
**Milestone:** `Sprint 1C - Organization Management`


**Description:**
This issue implements **1C-R2: Scope-aware naming regression coverage** in **Sprint 1C - Organization Management** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- The primary organization list title changes according to highest assigned scope when required.
- Scope-aware labels remain consistent across list, detail, and navigation surfaces.
- No route falls back to generic naming where scope-aware naming is required.

### 1C-R3: Organization CRUD and scoped-browse regression suite
**Scope:** QA / admin + product  
**Labels:** `phase:1-identity-access-forms`, `sprint:1C`, `type:test`, `status:regression-only`, `area:authorization`, `area:organization`, `area:admin`
**Depends on:** None
**Milestone:** `Sprint 1C - Organization Management`


**Description:**
This issue implements **1C-R3: Organization CRUD and scoped-browse regression suite** in **Sprint 1C - Organization Management** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can browse, create, edit, and inspect organization nodes through the app.
- Scoped users see the correct slice of the hierarchy.
- Detail and edit flows do not require workbench-only or ID-driven paths.

---

## Sprint 1D: Forms, Fields, And Version Authoring (Complete — regression backlog only)

### 1D-R1: Form builder CRUD regression suite
**Scope:** QA / forms  
**Labels:** `phase:1-identity-access-forms`, `sprint:1D`, `type:test`, `status:regression-only`, `area:forms`, `area:versioning`
**Depends on:** None
**Milestone:** `Sprint 1D - Forms, Fields, And Version Authoring`


**Description:**
This issue implements **1D-R1: Form builder CRUD regression suite** in **Sprint 1D - Forms, Fields, And Version Authoring** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can create, inspect, edit, and publish forms entirely through the app.
- Form detail and edit screens remain application-grade rather than builder fallback flows.
- Regressions identify whether the failure is form CRUD or version lifecycle behavior.

### 1D-R2: Field authoring regression suite
**Scope:** QA / form builder  
**Labels:** `phase:1-identity-access-forms`, `sprint:1D`, `type:test`, `status:regression-only`, `area:forms`, `area:versioning`
**Depends on:** None
**Milestone:** `Sprint 1D - Forms, Fields, And Version Authoring`


**Description:**
This issue implements **1D-R2: Field authoring regression suite** in **Sprint 1D - Forms, Fields, And Version Authoring** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Field creation, edit, delete, and reorder operations work through the intended UI.
- Option-set and lookup-source touchpoints remain reachable and usable.
- Published form versions preserve the intended field order and definitions.

### 1D-R3: Workflow attachment regression coverage
**Scope:** QA / integration  
**Labels:** `phase:1-identity-access-forms`, `sprint:1D`, `type:test`, `status:regression-only`, `area:workflows`, `area:forms`, `area:versioning`
**Depends on:** None
**Milestone:** `Sprint 1D - Forms, Fields, And Version Authoring`


**Description:**
This issue implements **1D-R3: Workflow attachment regression coverage** in **Sprint 1D - Forms, Fields, And Version Authoring** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Published form versions can still be attached at workflow integration points.
- Attachment failures surface as clear UI or validation errors.
- Regression output identifies whether the issue is publish, attachment, or display behavior.

---

## Sprint 1E: Form Semantic Versioning And Compatibility Automation

### 1E-01: Publish-time diff engine and semantic bump classifier
**Scope:** Backend / domain / forms  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:feature`, `status:backlog`, `area:backend`, `area:forms`, `area:versioning`
**Depends on:** None
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **Publish-time diff engine and semantic bump classifier** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation**. The primary goal is to add a publish-time structural diff that compares the draft revision against the current published baseline. It also includes work to classify the diff as `PATCH`, `MINOR`, or `MAJOR`. This ticket spans **Backend / domain / forms** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add a publish-time structural diff that compares the draft revision against the current published baseline.
- Classify the diff as `PATCH`, `MINOR`, or `MAJOR`.

**Acceptance criteria:**
- Publishing a revision computes a semantic bump without asking the user to type a version label.
- The classifier produces stable outcomes for identical inputs.
- Regression tests cover at least one patch-safe, one backward-compatible, and one breaking change case.

### 1E-02: Major-line compatibility resolver
**Scope:** Backend / domain / versioning  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`, `area:versioning`
**Depends on:** `1E-01`
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **Major-line compatibility resolver** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation**. The primary goal is to automatically reuse the current major line for compatible revisions. It also includes work to roll over to a new major line for breaking revisions. This ticket spans **Backend / domain / versioning** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Automatically reuse the current major line for compatible revisions.
- Roll over to a new major line for breaking revisions.

**Acceptance criteria:**
- Compatible publishes stay in the current major line.
- Breaking publishes create a new major line automatically.
- Users never enter a compatibility-group identifier manually.

### 1E-03: Downstream binding and stale-dependency detection
**Scope:** Backend / domain / impact analysis  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`, `area:forms`
**Depends on:** `1E-01`, `1E-02`
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **Downstream binding and stale-dependency detection** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation**. The primary goal is to bind datasets and direct form-bound report consumers to the current published form major. It also includes work to surface stale-dependency warnings when a breaking form change leaves downstream assets behind. This ticket spans **Backend / domain / impact analysis** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Bind datasets and direct form-bound report consumers to the current published form major.
- Surface stale-dependency warnings when a breaking form change leaves downstream assets behind.

**Acceptance criteria:**
- Existing compatible consumers remain bound without silent drift.
- Breaking revisions surface stale-dependency findings for affected downstream assets.
- Downstream impact is available to the publish review path.

### 1E-04: Publish review UI for semantic version and impact
**Scope:** Frontend / forms  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:feature`, `status:backlog`, `area:frontend`, `area:forms`, `area:versioning`
**Depends on:** `1E-01`, `1E-02`, `1E-03`
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **Publish review UI for semantic version and impact** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation**. The primary goal is to add a publish review screen that shows the proposed semantic version, major-line decision, and downstream impact. This ticket spans **Frontend / forms** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add a publish review screen that shows the proposed semantic version, major-line decision, and downstream impact.

**Acceptance criteria:**
- A form author can see the proposed semantic version before confirming publish.
- The UI clearly distinguishes same-major publishes from new-major publishes.
- Downstream impacts are readable without leaving the publish flow.

### 1E-05: Form detail and edit compatibility messaging
**Scope:** Frontend / forms  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:feature`, `status:backlog`, `area:frontend`, `area:forms`, `area:dependencies`
**Depends on:** `1E-02`, `1E-03`
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **Form detail and edit compatibility messaging** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation**. The primary goal is to show compatibility status and published-line history on form detail/edit surfaces. This ticket spans **Frontend / forms** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Show compatibility status and published-line history on form detail/edit surfaces.

**Acceptance criteria:**
- Form detail shows whether the latest publish stayed in the current major line or created a new one.
- Draft/edit surfaces surface the pending compatibility posture if enough information exists.
- Messaging uses product vocabulary rather than internal compatibility IDs.

### 1E-06: Semantic versioning UAT and regression suite
**Scope:** QA / forms / downstream impact  
**Labels:** `phase:1-identity-access-forms`, `sprint:1E`, `type:test`, `status:backlog`, `area:forms`, `area:versioning`, `area:dependencies`
**Depends on:** `1E-01`, `1E-02`, `1E-03`, `1E-04`, `1E-05`
**Milestone:** `Sprint 1E - Form Semantic Versioning And Compatibility Automation`


**Description:**
This issue implements **1E-06: Semantic versioning UAT and regression suite** in **Sprint 1E - Form Semantic Versioning And Compatibility Automation** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can revise, publish, and verify the assigned semantic version plus major-line outcome through the UI.
- At least one UI path demonstrates downstream stale-dependency warnings.
- Publish automation failures identify whether the issue is diffing, classification, line assignment, or UI display.

---

## Sprint 1F: Application UI Guidance Alignment (Complete — regression backlog only)

### 1F-R1: Shared-shell responsive regression suite
**Scope:** QA / frontend  
**Labels:** `phase:1-identity-access-forms`, `sprint:1F`, `type:test`, `status:regression-only`, `area:frontend`
**Depends on:** None
**Milestone:** `Sprint 1F - Application UI Guidance Alignment`


**Description:**
This issue implements **1F-R1: Shared-shell responsive regression suite** in **Sprint 1F - Application UI Guidance Alignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- The shared shell works at desktop and narrow widths without shell-level horizontal scroll.
- Top bar, sidebar, breadcrumbs, and spacing remain aligned to guidance.
- Regressions identify shell layout issues separately from page-content issues.

### 1F-R2: Product vs internal-area framing regression coverage
**Scope:** QA / IA / shell  
**Labels:** `phase:1-identity-access-forms`, `sprint:1F`, `type:test`, `status:regression-only`, `area:frontend`
**Depends on:** None
**Milestone:** `Sprint 1F - Application UI Guidance Alignment`


**Description:**
This issue implements **1F-R2: Product vs internal-area framing regression coverage** in **Sprint 1F - Application UI Guidance Alignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Administration remains subtle and Migration remains subordinate inside the shell.
- Product-facing surfaces do not drift back toward builder-era framing.
- The current page-family patterns remain consistent across major routes.

### 1F-R3: Hydration and browser-console baseline for core routes
**Scope:** QA / frontend quality  
**Labels:** `phase:1-identity-access-forms`, `sprint:1F`, `type:test`, `status:regression-only`, `area:frontend`
**Depends on:** None
**Milestone:** `Sprint 1F - Application UI Guidance Alignment`


**Description:**
This issue implements **1F-R3: Hydration and browser-console baseline for core routes** in **Sprint 1F - Application UI Guidance Alignment** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Home, Organization, Forms, Responses, Dashboards, Administration, and Migration are free of hydration regressions on their current implementation.
- Browser-console output is clean for those routes during sprint-close verification.
- Failures indicate the specific route that regressed.

---

## Sprint 1G: Tessara UI Component System Foundation (Complete — regression backlog only)

### 1G-R1: Component adoption audit
**Scope:** Frontend / architecture  
**Labels:** `phase:1-identity-access-forms`, `sprint:1G`, `type:refactor`, `status:regression-only`, `area:frontend`, `area:components`
**Depends on:** None
**Milestone:** `Sprint 1G - Tessara UI Component System Foundation`


**Description:**
This issue implements **1G-R1: Component adoption audit** in **Sprint 1G - Tessara UI Component System Foundation** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Touched shared routes use `tessara-ui` primitives where an approved component exists.
- Any route that still uses bespoke markup is listed with a reason and replacement target.
- New route work stops introducing a parallel visual system.

### 1G-R2: `tessara-ui` component contract examples
**Scope:** Frontend / documentation  
**Labels:** `phase:1-identity-access-forms`, `sprint:1G`, `type:docs`, `status:regression-only`, `area:frontend`, `area:components`, `area:documentation`
**Depends on:** None
**Milestone:** `Sprint 1G - Tessara UI Component System Foundation`


**Description:**
This issue implements **1G-R2: `tessara-ui` component contract examples** in **Sprint 1G - Tessara UI Component System Foundation** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Stable primitives have checked-in usage examples or thin test fixtures.
- Engineers can find the expected appearance and behavior without reading page-local markup.
- Component examples align with `ui-guidance.md` and style examples.

### 1G-R3: Guardrail against new bespoke shared-surface patterns
**Scope:** Frontend / code review policy  
**Labels:** `phase:1-identity-access-forms`, `sprint:1G`, `type:refactor`, `status:regression-only`, `area:frontend`, `area:components`
**Depends on:** None
**Milestone:** `Sprint 1G - Tessara UI Component System Foundation`


**Description:**
This issue implements **1G-R3: Guardrail against new bespoke shared-surface patterns** in **Sprint 1G - Tessara UI Component System Foundation** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- New shared-surface work uses `tessara-ui` when an approved primitive exists.
- Exceptions are documented in the PR or issue with a reason.
- At least one lightweight lint, review checklist, or test fixture supports the guardrail.

---

# Phase 2: Workflow Runtime, Responses, And Materialization

## Sprint 2A: Workflow Assignment And Response Start (Complete — regression backlog only)

### 2A-R1: Workflow assignment regression suite
**Scope:** QA / workflows  
**Labels:** `phase:2-runtime-responses`, `sprint:2A`, `type:test`, `status:regression-only`, `area:workflows`, `area:responses`
**Depends on:** None
**Milestone:** `Sprint 2A - Workflow Assignment And Response Start`


**Description:**
This issue implements **2A-R1: Workflow assignment regression suite** in **Sprint 2A - Workflow Assignment And Response Start** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can assign work through the product UI without builder tooling.
- Scope-aware visibility for assignment targets remains correct.
- Regression output distinguishes assignment creation from assignment display failures.

### 2A-R2: Response-start and pending-work regression suite
**Scope:** QA / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2A`, `type:test`, `status:regression-only`, `area:runtime`, `area:responses`, `area:workflows`
**Depends on:** None
**Milestone:** `Sprint 2A - Workflow Assignment And Response Start`


**Description:**
This issue implements **2A-R2: Response-start and pending-work regression suite** in **Sprint 2A - Workflow Assignment And Response Start** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Pending-work surfaces show the correct work items for the signed-in user.
- Starting a response enters the correct workflow/form path.
- Resume entry points continue to resolve to the correct in-progress work item.

### 2A-R3: Native SSR ownership regression for touched settled routes
**Scope:** QA / frontend  
**Labels:** `phase:2-runtime-responses`, `sprint:2A`, `type:test`, `status:regression-only`, `area:frontend`, `area:workflows`, `area:responses`
**Depends on:** None
**Milestone:** `Sprint 2A - Workflow Assignment And Response Start`


**Description:**
This issue implements **2A-R3: Native SSR ownership regression for touched settled routes** in **Sprint 2A - Workflow Assignment And Response Start** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- The Sprint 2A-touched `Home`, `Forms`, `Workflows`, and `Responses` surfaces render under native SSR ownership.
- Direct load, refresh, and in-app navigation do not fall back to the hybrid shell.
- Hydration and browser-console checks stay clean for those routes.

---

## Sprint 2B: Authentication Hardening And Settled-Surface Native SSR Slice

### 2B-01: Password-hash schema and migration/backfill
**Scope:** Backend / data / auth  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:backend`, `area:auth`, `area:migration`
**Depends on:** None
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Password-hash schema and migration/backfill** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to introduce Argon2id password-hash storage and any supporting metadata needed for versioning and future rotation. It also includes work to backfill seeded/demo accounts and update create/edit user flows so raw passwords are never stored. This ticket spans **Backend / data / auth** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Introduce Argon2id password-hash storage and any supporting metadata needed for versioning and future rotation.
- Backfill seeded/demo accounts and update create/edit user flows so raw passwords are never stored.

**Acceptance criteria:**
- New and updated accounts persist password hashes rather than raw passwords.
- Seeded/demo accounts can still authenticate after migration/backfill.
- Running the migration on an empty or populated local database is safe and repeatable.

### 2B-02: Auth verification service using Argon2id
**Scope:** Backend / auth  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:backend`, `area:auth`, `area:frontend`
**Depends on:** `2B-01`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Auth verification service using Argon2id** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to replace direct plaintext comparison with fetch-by-identity plus Argon2id verification. It also includes work to centralize auth verification logic so later session work uses one path. This ticket spans **Backend / auth** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Replace direct plaintext comparison with fetch-by-identity plus Argon2id verification.
- Centralize auth verification logic so later session work uses one path.

**Acceptance criteria:**
- Successful login verifies the password hash in application code.
- Failed login returns a stable auth error without leaking whether the hash, user, or query failed internally.
- Auth verification logic is covered by focused unit or integration tests.

### 2B-03: Server-managed browser session contract
**Scope:** Backend + frontend / auth  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:auth`
**Depends on:** `2B-02`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Server-managed browser session contract** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to move browser `/app` auth to same-origin cookie sessions with secure defaults. It also includes work to keep bearer tokens only for explicit script/CLI/testing flows. This ticket spans **Backend + frontend / auth** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Move browser `/app` auth to same-origin cookie sessions with secure defaults.
- Keep bearer tokens only for explicit script/CLI/testing flows.

**Acceptance criteria:**
- Browser-authenticated `/app` requests succeed without JavaScript-managed bearer tokens.
- Session cookies use `HttpOnly` and an appropriate `SameSite` policy, with secure settings for non-local environments.
- Scripted or CLI auth remains possible through an explicit non-browser flow.

### 2B-04: Session expiry, revocation, logout, and last-seen tracking
**Scope:** Backend / auth / sessions  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:backend`, `area:auth`, `area:frontend`
**Depends on:** `2B-03`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Session expiry, revocation, logout, and last-seen tracking** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to add expiry and revocation semantics to server-side sessions. It also includes work to update session activity metadata and make logout actively invalidate the current browser session. This ticket spans **Backend / auth / sessions** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add expiry and revocation semantics to server-side sessions.
- Update session activity metadata and make logout actively invalidate the current browser session.

**Acceptance criteria:**
- Expired or revoked sessions are rejected consistently.
- Logout invalidates the current session and returns the browser to an unauthenticated state.
- Session activity timestamps update in a controlled and testable way.

### 2B-05: Central authenticated-account extractor and request context
**Scope:** Backend / API architecture  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:refactor`, `status:backlog`, `area:backend`, `area:auth`, `area:frontend`
**Depends on:** `2B-02`, `2B-03`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Central authenticated-account extractor and request context** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to introduce a reusable authenticated-account extractor or request context boundary. It also includes work to stop repeating direct cookie/header parsing across handlers. This ticket spans **Backend / API architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Introduce a reusable authenticated-account extractor or request context boundary.
- Stop repeating direct cookie/header parsing across handlers.

**Acceptance criteria:**
- Touched handlers receive an authenticated principal/context through a shared boundary.
- Handlers no longer reimplement cookie or auth-header parsing.
- Auth failures map through one application-level error path.

### 2B-06: Stable auth/session error envelope and server tracing
**Scope:** Backend / observability / API  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:backend`, `area:auth`, `area:observability`
**Depends on:** `2B-02`, `2B-03`, `2B-04`, `2B-05`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Stable auth/session error envelope and server tracing** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to replace raw internal/database error exposure with stable auth/session codes and messages. It also includes work to improve server logs so failed auth/session flows remain traceable without leaking internal detail to clients. This ticket spans **Backend / observability / API** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Replace raw internal/database error exposure with stable auth/session codes and messages.
- Improve server logs so failed auth/session flows remain traceable without leaking internal detail to clients.

**Acceptance criteria:**
- End users do not receive raw database or internal error strings for auth/session failures.
- Server logs retain enough detail to debug failures with correlation or trace context.
- At least one test proves the client-visible payload is stable even when the server log contains richer detail.

### 2B-07: Native SSR login and home surfaces
**Scope:** Frontend / auth / shell  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:frontend`, `area:auth`
**Depends on:** `2B-03`, `2B-05`, `2B-06`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Native SSR login and home surfaces** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to migrate `/app/login` and `/app` to native SSR ownership. It also includes work to remove dependency on the retained bridge for normal login, sign-in redirect, refresh, and sign-out behavior. This ticket spans **Frontend / auth / shell** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Migrate `/app/login` and `/app` to native SSR ownership.
- Remove dependency on the retained bridge for normal login, sign-in redirect, refresh, and sign-out behavior.

**Acceptance criteria:**
- Login, reload, post-login entry, and sign-out work without the hybrid shell.
- SSR render and hydration are clean on login and home routes.
- There is no visible dependency on shipped demo passwords in the login surface.

### 2B-08: Native SSR Organization and Forms settled routes
**Scope:** Frontend / product routes  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:feature`, `status:backlog`, `area:frontend`, `area:organization`, `area:forms`
**Depends on:** `2B-03`, `2B-05`, `2B-06`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **Native SSR Organization and Forms settled routes** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice**. The primary goal is to migrate `/app/organization*` and `/app/forms*` onto native SSR ownership. It also includes work to replace any route-local bridge state used by these settled surfaces. This ticket spans **Frontend / product routes** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Migrate `/app/organization*` and `/app/forms*` onto native SSR ownership.
- Replace any route-local bridge state used by these settled surfaces.

**Acceptance criteria:**
- Organization and Forms list/detail/edit/publish flows work without the retained hybrid shell.
- Direct navigation and refresh on those routes remain stable.
- Touched shared UI uses component primitives and avoids raw inline action handlers.

### 2B-09: Auth hardening UAT and regression matrix
**Scope:** QA / auth / product  
**Labels:** `phase:2-runtime-responses`, `sprint:2B`, `type:test`, `status:backlog`, `area:auth`, `area:frontend`
**Depends on:** `2B-01`, `2B-02`, `2B-03`, `2B-04`, `2B-05`, `2B-06`, `2B-07`, `2B-08`
**Milestone:** `Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice`


**Description:**
This issue implements **2B-09: Auth hardening UAT and regression matrix** in **Sprint 2B - Authentication Hardening And Settled-Surface Native SSR Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can sign in, refresh, browse Organization and Forms, edit/publish a form, and sign out under the new session contract.
- Expired or revoked sessions fail safely in both UI and API behavior.
- Sprint-close output includes route ownership plus hydration/browser-console cleanliness for login, home, Organization, and Forms.

---

## Sprint 2C: Workflow/Response Route Ownership And Backend Decomposition Slice

### 2C-01: Native SSR workflow browse/detail/assignment surfaces
**Scope:** Frontend / workflows  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:feature`, `status:backlog`, `area:frontend`, `area:workflows`, `area:responses`
**Depends on:** `2B-09`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Native SSR workflow browse/detail/assignment surfaces** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to migrate `/app/workflows*` assignment and browse/detail flows onto native SSR ownership. It also includes work to remove route-local bridge dependencies from these settled workflow surfaces. This ticket spans **Frontend / workflows** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Migrate `/app/workflows*` assignment and browse/detail flows onto native SSR ownership.
- Remove route-local bridge dependencies from these settled workflow surfaces.

**Acceptance criteria:**
- Workflow browse/detail/assignment routes operate without the retained hybrid shell.
- Reload and in-app navigation preserve the selected workflow context correctly.
- Browser-console and hydration checks are clean for touched workflow routes.

### 2C-02: Native SSR response-start and resume entry surfaces
**Scope:** Frontend / responses / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:feature`, `status:backlog`, `area:frontend`, `area:runtime`, `area:responses`
**Depends on:** `2B-09`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Native SSR response-start and resume entry surfaces** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to migrate response-start, pending-work, and resume entry routes onto native SSR ownership. It also includes work to keep deep response editing/review for the next sprint, but own the entry surfaces natively now. This ticket spans **Frontend / responses / runtime** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Migrate response-start, pending-work, and resume entry routes onto native SSR ownership.
- Keep deep response editing/review for the next sprint, but own the entry surfaces natively now.

**Acceptance criteria:**
- A tester can start or resume the correct response flow without falling back to the bridge.
- Pending-work and entry pages hydrate cleanly and reload correctly.
- Entry surfaces use the settled browser auth/session contract from Sprint 2B.

### 2C-03: Decompose touched backend auth/hierarchy/forms modules
**Scope:** Backend / architecture  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:refactor`, `status:backlog`, `area:backend`, `area:auth`, `area:organization`
**Depends on:** `2B-05`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Decompose touched backend auth/hierarchy/forms modules** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to split touched auth, hierarchy, and forms code toward `router`, `handlers`, `service`, `repo`, and `dto`. It also includes work to keep business rules out of `lib.rs`. This ticket spans **Backend / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Split touched auth, hierarchy, and forms code toward `router`, `handlers`, `service`, `repo`, and `dto`.
- Keep business rules out of `lib.rs`.

**Acceptance criteria:**
- Newly touched auth/hierarchy/forms behavior lands outside giant vertical files when feasible.
- `tessara-api::lib` remains a composition root rather than a business-logic host.
- New tests can target service or repo layers without booting unrelated route code.

### 2C-04: Decompose touched workflow/submission modules
**Scope:** Backend / architecture  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:refactor`, `status:backlog`, `area:backend`, `area:workflows`, `area:responses`
**Depends on:** `2C-03`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Decompose touched workflow/submission modules** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to apply the same module split to touched workflow and submissions code. It also includes work to move orchestration into services and SQL into repositories. This ticket spans **Backend / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Apply the same module split to touched workflow and submissions code.
- Move orchestration into services and SQL into repositories.

**Acceptance criteria:**
- Touched workflow/submission handlers are thin enough to primarily decode input, call services, and shape responses.
- SQL or DB orchestration no longer expands inside route handlers for touched flows.
- The changed structure is documented enough for follow-on tickets to use consistently.

### 2C-05: Shrink `tessara-api::lib` to routing/middleware/state composition
**Scope:** Backend / architecture  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:refactor`, `status:backlog`, `area:backend`, `area:workflows`, `area:responses`
**Depends on:** `2C-03`, `2C-04`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Shrink `tessara-api::lib` to routing/middleware/state composition** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to ensure no new business workflows are added to `lib.rs`. It also includes work to move route registration and state setup into clearer composition helpers where useful. This ticket spans **Backend / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Ensure no new business workflows are added to `lib.rs`.
- Move route registration and state setup into clearer composition helpers where useful.

**Acceptance criteria:**
- `lib.rs` primarily wires routers, middleware, state, and shared configuration.
- New business logic does not land directly in `lib.rs`.
- Route composition remains easy to audit after the change.

### 2C-06: Tighten shared UI primitives for SSR-owned settled routes
**Scope:** Frontend / component system  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:feature`, `status:backlog`, `area:frontend`, `area:components`, `area:workflows`
**Depends on:** `2B-08`, `1G-R1`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **Tighten shared UI primitives for SSR-owned settled routes** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice**. The primary goal is to replace raw inline `onclick` patterns on newly migrated routes with safer component/event patterns. It also includes work to update touched `tessara-ui` primitives if needed. This ticket spans **Frontend / component system** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Replace raw inline `onclick` patterns on newly migrated routes with safer component/event patterns.
- Update touched `tessara-ui` primitives if needed.

**Acceptance criteria:**
- Newly migrated SSR routes do not rely on raw inline action-handler strings.
- Shared action primitives remain usable across settled routes after the change.
- Any remaining inline-handler exceptions are isolated and explicitly tracked.

### 2C-07: Targeted integration suites for settled workflow/runtime flows
**Scope:** QA / backend + frontend  
**Labels:** `phase:2-runtime-responses`, `sprint:2C`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:workflows`
**Depends on:** `2C-01`, `2C-02`, `2C-03`, `2C-04`, `2C-05`, `2C-06`
**Milestone:** `Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice`


**Description:**
This issue implements **2C-07: Targeted integration suites for settled workflow/runtime flows** in **Sprint 2C - Workflow/Response Route Ownership And Backend Decomposition Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- There are focused integration suites for auth/session, role/capability boundaries, form publish safeguards, workflow assignment, and response-start behavior.
- Failing tests isolate the broken bounded context instead of only failing a giant demo script.
- Sprint-close verification includes native-route ownership for Workflows and Responses entry surfaces.

---

## Sprint 2D: Draft, Submit, And Review Response Slice

### 2D-01: Response draft persistence service
**Scope:** Backend / submissions / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:feature`, `status:backlog`, `area:backend`, `area:runtime`, `area:responses`
**Depends on:** `2C-07`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **Response draft persistence service** in **Sprint 2D - Draft, Submit, And Review Response Slice**. The primary goal is to establish canonical draft persistence behavior for in-progress responses. It also includes work to ensure save/resume flows use one domain service path. This ticket spans **Backend / submissions / runtime** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Establish canonical draft persistence behavior for in-progress responses.
- Ensure save/resume flows use one domain service path.

**Acceptance criteria:**
- Draft saves persist through the application flow without ad hoc side paths.
- Resuming a draft restores the expected answer state.
- Draft persistence is covered by focused service or integration tests.

### 2D-02: Native SSR response editing surfaces
**Scope:** Frontend / responses  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:feature`, `status:backlog`, `area:frontend`, `area:responses`
**Depends on:** `2D-01`, `2B-09`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **Native SSR response editing surfaces** in **Sprint 2D - Draft, Submit, And Review Response Slice**. The primary goal is to deliver response edit routes as native SSR from first release. It also includes work to do not add any bridge fallback for edit behavior. This ticket spans **Frontend / responses** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver response edit routes as native SSR from first release.
- Do not add any bridge fallback for edit behavior.

**Acceptance criteria:**
- A tester can open and edit in-progress responses through SSR-owned routes.
- Reload on an edit route restores the correct response context.
- Touched response routes remain hydration-clean and console-clean.

### 2D-03: Save and strict submit behavior
**Scope:** Backend + frontend / submissions  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:responses`
**Depends on:** `2D-01`, `2D-02`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **Save and strict submit behavior** in **Sprint 2D - Draft, Submit, And Review Response Slice**. The primary goal is to implement explicit save and submit paths with validation and state transitions. It also includes work to prevent invalid or duplicate submit behavior. This ticket spans **Backend + frontend / submissions** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Implement explicit save and submit paths with validation and state transitions.
- Prevent invalid or duplicate submit behavior.

**Acceptance criteria:**
- Save and submit are separate user actions with clear outcomes.
- Invalid submissions fail with stable, user-readable errors.
- Once submitted, the response cannot silently remain editable through draft-only paths.

### 2D-04: Read-only submitted review surfaces
**Scope:** Frontend / responses  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:feature`, `status:backlog`, `area:frontend`, `area:responses`
**Depends on:** `2D-03`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **Read-only submitted review surfaces** in **Sprint 2D - Draft, Submit, And Review Response Slice**. The primary goal is to provide coherent read-only review routes for submitted responses. This ticket spans **Frontend / responses** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Provide coherent read-only review routes for submitted responses.

**Acceptance criteria:**
- Submitted responses render in a clearly read-only state.
- Review surfaces show the final submitted content rather than an editable draft representation.
- Reload and deep-link to a submitted response remain stable.

### 2D-05: Submissions backend decomposition continuation
**Scope:** Backend / architecture  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:refactor`, `status:backlog`, `area:backend`, `area:responses`
**Depends on:** `2C-04`, `2D-01`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **Submissions backend decomposition continuation** in **Sprint 2D - Draft, Submit, And Review Response Slice**. The primary goal is to continue the `handler`, `service`, and `repo` split introduced in Sprint 2C for submissions and runtime work touched here. This ticket spans **Backend / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Continue the `handler`, `service`, and `repo` split introduced in Sprint 2C for submissions and runtime work touched here.

**Acceptance criteria:**
- New response-lifecycle orchestration lives in services rather than handlers.
- Repository code owns persistence concerns for touched paths.
- The response lifecycle can be tested through narrower layers than a whole-app demo path.

### 2D-06: Response lifecycle UAT matrix
**Scope:** QA / responses  
**Labels:** `phase:2-runtime-responses`, `sprint:2D`, `type:test`, `status:backlog`, `area:responses`
**Depends on:** `2D-01`, `2D-02`, `2D-03`, `2D-04`, `2D-05`
**Milestone:** `Sprint 2D - Draft, Submit, And Review Response Slice`


**Description:**
This issue implements **2D-06: Response lifecycle UAT matrix** in **Sprint 2D - Draft, Submit, And Review Response Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can save draft, resume, submit, and review responses through the application UI.
- The same tester cannot keep editing through draft paths after submit unless a deliberate feature says so.
- Sprint-close output proves the touched response lifecycle routes remain under native SSR ownership.

---

## Sprint 2E: Multi-Step Workflow Authoring And Execution

### 2E-01: Multi-step workflow version data model and publish validation
**Scope:** Backend / workflows / domain  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:backend`, `area:workflows`, `area:versioning`
**Depends on:** `2D-06`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Multi-step workflow version data model and publish validation** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to add ordered step definitions to workflow versions. It also includes work to enforce publish-time structural completeness. This ticket spans **Backend / workflows / domain** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add ordered step definitions to workflow versions.
- Enforce publish-time structural completeness.

**Acceptance criteria:**
- A workflow version can define more than one ordered step.
- Publish fails with stable validation errors if required step structure is incomplete.
- The model is ready for runtime progression without replacing the 2A/2D foundation.

### 2E-02: Workflow authoring UI for multi-step versions
**Scope:** Frontend / workflows  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:frontend`, `area:workflows`
**Depends on:** `2E-01`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Workflow authoring UI for multi-step versions** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to deliver authoring screens that let operators define and inspect ordered workflow steps. This ticket spans **Frontend / workflows** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver authoring screens that let operators define and inspect ordered workflow steps.

**Acceptance criteria:**
- Operators can create and edit multi-step workflow versions in the app.
- Step order is visible and editable through the authoring UI.
- The authoring routes remain native SSR and do not reintroduce bridge-owned state.

### 2E-03: Runtime progression service across workflow steps
**Scope:** Backend / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:backend`, `area:workflows`, `area:runtime`
**Depends on:** `2E-01`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Runtime progression service across workflow steps** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to add runtime logic that advances workflow instances from one step to the next. This ticket spans **Backend / runtime** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add runtime logic that advances workflow instances from one step to the next.

**Acceptance criteria:**
- Completing one step activates the correct next step.
- Runtime state changes are persisted through a central service path.
- Failed progression attempts surface stable, workflow-aware errors.

### 2E-04: Step-specific assignment and pending-work behavior
**Scope:** Backend + frontend / workflows  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:workflows`
**Depends on:** `2E-01`, `2E-03`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Step-specific assignment and pending-work behavior** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to support assignment at the step level and reflect that in pending-work surfaces. This ticket spans **Backend + frontend / workflows** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Support assignment at the step level and reflect that in pending-work surfaces.

**Acceptance criteria:**
- Assignment can target the current active step rather than only the whole workflow.
- Pending-work lists show the right active step for the right assignee.
- Completing the current step updates pending-work visibility correctly.

### 2E-05: Step handoff history and runtime visibility
**Scope:** Frontend / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:frontend`, `area:runtime`, `area:workflows`
**Depends on:** `2E-03`, `2E-04`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Step handoff history and runtime visibility** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to show current step, upcoming step, and completed-step history for in-flight work. This ticket spans **Frontend / runtime** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Show current step, upcoming step, and completed-step history for in-flight work.

**Acceptance criteria:**
- Runtime surfaces clearly indicate the current active step.
- Completed steps are visible as history, not merged into the active-step state.
- The next step becomes visible at the appropriate handoff point.

### 2E-06: Typed workflow step and runtime states
**Scope:** Backend / domain typing  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:feature`, `status:backlog`, `area:backend`, `area:workflows`, `area:runtime`
**Depends on:** `2E-01`, `2E-03`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **Typed workflow step and runtime states** in **Sprint 2E - Multi-Step Workflow Authoring And Execution**. The primary goal is to replace new raw string expansion for workflow step/runtime state with typed values where touched. This ticket spans **Backend / domain typing** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Replace new raw string expansion for workflow step/runtime state with typed values where touched.

**Acceptance criteria:**
- Newly added multi-step states are represented as typed values in the touched domain/service code.
- Request/response boundaries map to those typed values predictably.
- Tests cover at least one invalid-state transition attempt.

### 2E-07: Multi-step workflow execution UAT
**Scope:** QA / workflows / runtime  
**Labels:** `phase:2-runtime-responses`, `sprint:2E`, `type:test`, `status:backlog`, `area:workflows`, `area:runtime`
**Depends on:** `2E-01`, `2E-02`, `2E-03`, `2E-04`, `2E-05`, `2E-06`
**Milestone:** `Sprint 2E - Multi-Step Workflow Authoring And Execution`


**Description:**
This issue implements **2E-07: Multi-step workflow execution UAT** in **Sprint 2E - Multi-Step Workflow Authoring And Execution** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can create a multi-step workflow, assign it, start it, complete the first step, and observe the next step become active.
- The runtime surfaces remain under native SSR ownership.
- Validation, assignment, and progression failures identify their bounded context clearly.

---

## Sprint 2F: Runtime Status And Materialization Slice

### 2F-01: Runtime status model and APIs
**Scope:** Backend / runtime / operators  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:feature`, `status:backlog`, `area:backend`, `area:runtime`, `area:operators`
**Depends on:** `2E-07`
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **Runtime status model and APIs** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to expose workflow/runtime status information through a stable internal model and API surface. This ticket spans **Backend / runtime / operators** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Expose workflow/runtime status information through a stable internal model and API surface.

**Acceptance criteria:**
- Operators can query runtime status without reading raw DB state.
- Status payloads use stable application fields and error semantics.
- The model is consistent enough for UI and tests to consume directly.

### 2F-02: Materialization readiness and refresh status
**Scope:** Backend / data pipeline  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:feature`, `status:backlog`, `area:backend`, `area:data-pipeline`, `area:runtime`
**Depends on:** `2E-07`
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **Materialization readiness and refresh status** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to surface materialization readiness and refresh status in a usable way for operators. This ticket spans **Backend / data pipeline** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Surface materialization readiness and refresh status in a usable way for operators.

**Acceptance criteria:**
- Materialization state distinguishes ready, stale, refreshing, and failed conditions if those states exist.
- Refresh or readiness failures surface stable messages rather than internal traces.
- Status data is available to the operator UI without ad hoc DB queries.

### 2F-03: Native SSR runtime/materialization monitoring screens
**Scope:** Frontend / operators  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:feature`, `status:backlog`, `area:frontend`, `area:runtime`, `area:data-pipeline`
**Depends on:** `2F-01`, `2F-02`
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **Native SSR runtime/materialization monitoring screens** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to provide coherent internal monitoring surfaces that do not disrupt the main user shell. This ticket spans **Frontend / operators** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Provide coherent internal monitoring surfaces that do not disrupt the main user shell.

**Acceptance criteria:**
- Operators can inspect runtime and materialization readiness through SSR-owned screens.
- Touched operator routes remain hydration-clean and console-clean.
- Monitoring routes clearly distinguish runtime status from end-user response flows.

### 2F-04: CI enforcement for documented quality checks
**Scope:** CI / repo hygiene  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:ops`, `status:backlog`, `area:ci`, `area:runtime`, `area:data-pipeline`
**Depends on:** None
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **CI enforcement for documented quality checks** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to enforce the documented checks: `fmt`, `check`, wasm hydrate check, `test`, `clippy`, smoke, and legacy import rehearsal. This ticket spans **CI / repo hygiene** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Enforce the documented checks: `fmt`, `check`, wasm hydrate check, `test`, `clippy`, smoke, and legacy import rehearsal.

**Acceptance criteria:**
- The required checks run automatically in CI or a clearly automated gate.
- Pull requests cannot silently bypass the documented quality bar.
- Failures identify which quality gate regressed.

### 2F-05: Split maintenance/import/demo commands from HTTP startup
**Scope:** Backend / ops / CLI  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:ops`, `status:backlog`, `area:backend`, `area:migration`, `area:runtime`
**Depends on:** None
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **Split maintenance/import/demo commands from HTTP startup** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to move maintenance, import, and demo commands out of the normal HTTP startup path. This ticket spans **Backend / ops / CLI** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Move maintenance, import, and demo commands out of the normal HTTP startup path.

**Acceptance criteria:**
- Starting the web server no longer conflates HTTP service startup with maintenance/demo operations.
- Maintenance/import/demo commands remain available through a separate CLI path.
- Local and CI docs are updated to show the new entry points.

### 2F-06: Workflow-aware tracing and operator-facing error hardening
**Scope:** Observability / backend  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:feature`, `status:backlog`, `area:backend`, `area:workflows`, `area:operators`
**Depends on:** `2F-01`, `2F-02`
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **Workflow-aware tracing and operator-facing error hardening** in **Sprint 2F - Runtime Status And Materialization Slice**. The primary goal is to add tracing around runtime/materialization flows and ensure operator-facing errors stay stable. This ticket spans **Observability / backend** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add tracing around runtime/materialization flows and ensure operator-facing errors stay stable.

**Acceptance criteria:**
- Runtime/materialization actions emit traceable server events with enough context to diagnose failures.
- End users/operators do not receive raw internal or database strings.
- At least one failure path demonstrates stable client messaging plus richer server logs.

### 2F-07: Runtime/materialization UAT closeout
**Scope:** QA / operators  
**Labels:** `phase:2-runtime-responses`, `sprint:2F`, `type:test`, `status:backlog`, `area:runtime`, `area:data-pipeline`, `area:operators`
**Depends on:** `2F-01`, `2F-02`, `2F-03`, `2F-04`, `2F-05`, `2F-06`
**Milestone:** `Sprint 2F - Runtime Status And Materialization Slice`


**Description:**
This issue implements **2F-07: Runtime/materialization UAT closeout** in **Sprint 2F - Runtime Status And Materialization Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Operators can inspect runtime and readiness through the app while end-user flows remain intact.
- Sprint-close verification explicitly includes hydration/browser-console cleanliness for touched internal routes.
- CI status and manual UAT agree on the touched route ownership.

---

# Phase 3: Dataset Engine And Revisions

## Sprint 3A: Dataset Authoring Slice

### 3A-01: Dataset bounded-context backend structure
**Scope:** Backend / architecture  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:refactor`, `status:backlog`, `area:backend`, `area:datasets`
**Depends on:** `2C-05`, `2F-04`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Dataset bounded-context backend structure** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to organize dataset work into clearer route, service, repo, and DTO boundaries. It also includes work to keep dataset query planning and execution behind explicit service boundaries. This ticket spans **Backend / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Organize dataset work into clearer route, service, repo, and DTO boundaries.
- Keep dataset query planning and execution behind explicit service boundaries.

**Acceptance criteria:**
- New dataset authoring work lands outside a single god-file pattern.
- Dataset handlers are thin enough to mostly decode, delegate, and shape responses.
- Query planning/execution concerns are reachable through explicit dataset/reporting services.

### 3A-02: Dataset directory/detail/create/edit SSR surfaces
**Scope:** Frontend / datasets  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:feature`, `status:backlog`, `area:frontend`, `area:datasets`
**Depends on:** `3A-01`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Dataset directory/detail/create/edit SSR surfaces** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to deliver application-grade dataset directory, detail, create, and edit flows. This ticket spans **Frontend / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver application-grade dataset directory, detail, create, and edit flows.

**Acceptance criteria:**
- A tester can create, inspect, and edit datasets through SSR-owned app routes.
- Dataset routes deep-link and reload cleanly.
- Touched routes do not revive legacy bridge ownership.

### 3A-03: Source composition authoring
**Scope:** Backend + frontend / datasets  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:datasets`
**Depends on:** `3A-01`, `3A-02`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Source composition authoring** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to support authoring of dataset source composition in the application. This ticket spans **Backend + frontend / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Support authoring of dataset source composition in the application.

**Acceptance criteria:**
- Dataset authors can select or configure multiple sources where supported.
- Source configuration is persisted through application flows, not ad hoc admin-only tooling.
- Invalid compositions fail with stable, user-readable validation messages.

### 3A-04: Row filters and calculated fields
**Scope:** Backend + frontend / datasets  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:forms`
**Depends on:** `3A-01`, `3A-03`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Row filters and calculated fields** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to add authoring paths for dataset row filters and calculated fields. This ticket spans **Backend + frontend / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add authoring paths for dataset row filters and calculated fields.

**Acceptance criteria:**
- Authors can define, edit, and remove row filters and calculated fields through the app.
- Preview or validation shows when a filter or calculated field is invalid.
- The saved dataset definition matches what the preview executes.

### 3A-05: Dataset preview with execution guardrails
**Scope:** Backend + frontend / datasets  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:datasets`
**Depends on:** `3A-03`, `3A-04`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Dataset preview with execution guardrails** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to provide preview execution while adding pagination, row limits, and time/size guardrails. This ticket spans **Backend + frontend / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Provide preview execution while adding pagination, row limits, and time/size guardrails.

**Acceptance criteria:**
- Preview results are visible in the app for editable datasets.
- Preview execution respects explicit row or execution limits.
- Timeout or size failures surface stable errors instead of unbounded or silent failures.

### 3A-06: Reporting execution guardrails on touched surfaces
**Scope:** Backend / reporting  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:feature`, `status:backlog`, `area:backend`, `area:reporting`, `area:datasets`
**Depends on:** `3A-01`, `3A-05`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **Reporting execution guardrails on touched surfaces** in **Sprint 3A - Dataset Authoring Slice**. The primary goal is to add or tighten limits on reporting/list surfaces touched by dataset authoring work. This ticket spans **Backend / reporting** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add or tighten limits on reporting/list surfaces touched by dataset authoring work.

**Acceptance criteria:**
- Touched reporting endpoints expose pagination or bounded result sizes.
- Large result sets do not require pulling the whole result set into memory just to render a page of UI where avoidable.
- Guardrail behavior is covered by tests.

### 3A-07: Dataset authoring UAT
**Scope:** QA / datasets  
**Labels:** `phase:3-datasets`, `sprint:3A`, `type:test`, `status:backlog`, `area:datasets`
**Depends on:** `3A-01`, `3A-02`, `3A-03`, `3A-04`, `3A-05`, `3A-06`
**Milestone:** `Sprint 3A - Dataset Authoring Slice`


**Description:**
This issue implements **3A-07: Dataset authoring UAT** in **Sprint 3A - Dataset Authoring Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can create, inspect, edit, and preview datasets through the application.
- Dataset routes remain native SSR and clean in hydration/browser-console checks.
- Preview and validation failures clearly identify dataset-authoring issues rather than generic server failures.

---

## Sprint 3B: Dataset Revision And Compatibility Slice

### 3B-01: Immutable dataset revision model and history
**Scope:** Backend / datasets / versioning  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:backend`, `area:datasets`, `area:versioning`
**Depends on:** `3A-07`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Immutable dataset revision model and history** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to add immutable dataset revisions and expose revision history. This ticket spans **Backend / datasets / versioning** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Add immutable dataset revisions and expose revision history.

**Acceptance criteria:**
- Publishing or revising a dataset creates a durable revision record.
- Revision history is queryable without reading internal DB tables directly.
- Revision history distinguishes draft/edit state from published revisions where applicable.

### 3B-02: Revision publishing flow
**Scope:** Backend + frontend / datasets  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:datasets`
**Depends on:** `3B-01`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Revision publishing flow** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to deliver the UI and backend flow for publishing dataset revisions. This ticket spans **Backend + frontend / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver the UI and backend flow for publishing dataset revisions.

**Acceptance criteria:**
- A tester can publish a dataset revision through the application.
- The published revision becomes visible in revision history and detail screens.
- Publish validation failures surface stable, revision-aware errors.

### 3B-03: Compatibility findings engine
**Scope:** Backend / compatibility analysis  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`, `area:datasets`
**Depends on:** `3B-01`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Compatibility findings engine** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to compute compatibility findings between revisions and surface their downstream impact. This ticket spans **Backend / compatibility analysis** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Compute compatibility findings between revisions and surface their downstream impact.

**Acceptance criteria:**
- A revised dataset shows whether the change is compatible or breaking according to the chosen rules.
- Compatibility findings are available to UI and downstream dependency checks through a typed contract.
- Tests cover at least one compatible and one incompatible revision scenario.

### 3B-04: Dependency visibility and typed contracts
**Scope:** Backend / dependency graph  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`, `area:datasets`
**Depends on:** `3B-01`, `3B-03`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Dependency visibility and typed contracts** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to surface downstream dependency visibility through typed contracts that later component/dashboard work can consume. This ticket spans **Backend / dependency graph** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Surface downstream dependency visibility through typed contracts that later component/dashboard work can consume.

**Acceptance criteria:**
- UI and services consume typed dependency results rather than ad hoc string blobs.
- Downstream assets affected by a revision are visible in the application.
- Dependency payloads are stable enough to use as inputs to publication guards later.

### 3B-05: Carry-forward behavior
**Scope:** Backend + frontend / revisions  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:datasets`
**Depends on:** `3B-01`, `3B-03`, `3B-04`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Carry-forward behavior** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to support carry-forward behavior for compatible revisions. This ticket spans **Backend + frontend / revisions** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Support carry-forward behavior for compatible revisions.

**Acceptance criteria:**
- Compatible downstream relationships can carry forward automatically where the product rule says they should.
- The UI distinguishes automatic carry-forward from blocked or warned relationships.
- Tests cover both carry-forward and non-carry-forward paths.

### 3B-06: Typed revision, compatibility, and dependency states
**Scope:** Backend / domain typing  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:feature`, `status:backlog`, `area:backend`, `area:datasets`, `area:dependencies`
**Depends on:** `3B-01`, `3B-03`, `3B-04`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **Typed revision, compatibility, and dependency states** in **Sprint 3B - Dataset Revision And Compatibility Slice**. The primary goal is to normalize revision, compatibility, and dependency states into typed values rather than raw string expansion. This ticket spans **Backend / domain typing** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Normalize revision, compatibility, and dependency states into typed values rather than raw string expansion.

**Acceptance criteria:**
- Touched service/domain code uses typed values for revision and compatibility outcomes.
- DB or API boundaries map string/storage values into typed values in one place.
- Invalid or unknown states fail predictably in tests.

### 3B-07: Dataset revision UAT
**Scope:** QA / datasets  
**Labels:** `phase:3-datasets`, `sprint:3B`, `type:test`, `status:backlog`, `area:datasets`, `area:dependencies`
**Depends on:** `3B-01`, `3B-02`, `3B-03`, `3B-04`, `3B-05`, `3B-06`
**Milestone:** `Sprint 3B - Dataset Revision And Compatibility Slice`


**Description:**
This issue implements **3B-07: Dataset revision UAT** in **Sprint 3B - Dataset Revision And Compatibility Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can revise a dataset, publish the revision, inspect compatibility findings, and understand downstream impact from the UI.
- Revision history/detail screens remain SSR-owned and clean in route checks.
- Failures identify whether the issue is publish, compatibility, dependency visibility, or carry-forward behavior.

---

# Phase 4: Components

## Sprint 4A: Table Component Slice

### 4A-01: Component domain model and versioning foundation
**Scope:** Backend / components  
**Labels:** `phase:4-components`, `sprint:4A`, `type:feature`, `status:backlog`, `area:backend`, `area:components`, `area:versioning`
**Depends on:** `3B-07`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **Component domain model and versioning foundation** in **Sprint 4A - Table Component Slice**. The primary goal is to introduce `Component` and `ComponentVersion` as the primary analytical presentation model for table components. This ticket spans **Backend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Introduce `Component` and `ComponentVersion` as the primary analytical presentation model for table components.

**Acceptance criteria:**
- Table component work persists through `Component` / `ComponentVersion` rather than new `Report/Aggregation/Chart` core entities.
- Component version records can be drafted, published, and inspected.
- The model is ready for future visual components without rework.

### 4A-02: Component directory/detail/create/edit/publish SSR surfaces
**Scope:** Frontend / components  
**Labels:** `phase:4-components`, `sprint:4A`, `type:feature`, `status:backlog`, `area:frontend`, `area:components`
**Depends on:** `4A-01`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **Component directory/detail/create/edit/publish SSR surfaces** in **Sprint 4A - Table Component Slice**. The primary goal is to deliver application-grade component directory, detail, create, edit, and publish routes. This ticket spans **Frontend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver application-grade component directory, detail, create, edit, and publish routes.

**Acceptance criteria:**
- A tester can create, inspect, edit, and publish table components through SSR-owned routes.
- Component routes reload and deep-link cleanly.
- Touched routes do not introduce new bridge ownership.

### 4A-03: `DetailTable` authoring and viewer
**Scope:** Backend + frontend / components  
**Labels:** `phase:4-components`, `sprint:4A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:components`
**Depends on:** `4A-01`, `4A-02`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **`DetailTable` authoring and viewer** in **Sprint 4A - Table Component Slice**. The primary goal is to implement authoring and viewing for `DetailTable`. This ticket spans **Backend + frontend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Implement authoring and viewing for `DetailTable`.

**Acceptance criteria:**
- Authors can configure a detail table against a dataset revision.
- Viewers can render the published table component in the application.
- Validation errors identify invalid dataset bindings or table configuration problems clearly.

### 4A-04: `AggregateTable` authoring and viewer
**Scope:** Backend + frontend / components  
**Labels:** `phase:4-components`, `sprint:4A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:components`
**Depends on:** `4A-01`, `4A-02`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **`AggregateTable` authoring and viewer** in **Sprint 4A - Table Component Slice**. The primary goal is to implement authoring and viewing for `AggregateTable`. This ticket spans **Backend + frontend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Implement authoring and viewing for `AggregateTable`.

**Acceptance criteria:**
- Authors can configure aggregate table behavior and publish it.
- Published aggregate tables render through the application viewer.
- Aggregate validation and preview behavior remain bounded and testable.

### 4A-05: Dataset-revision binding and validation behavior
**Scope:** Backend / components / datasets  
**Labels:** `phase:4-components`, `sprint:4A`, `type:feature`, `status:backlog`, `area:backend`, `area:datasets`, `area:components`
**Depends on:** `4A-01`, `3B-07`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **Dataset-revision binding and validation behavior** in **Sprint 4A - Table Component Slice**. The primary goal is to bind component versions to dataset revisions with explicit validation outcomes. This ticket spans **Backend / components / datasets** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Bind component versions to dataset revisions with explicit validation outcomes.

**Acceptance criteria:**
- A component version records the dataset revision it depends on.
- Binding failures surface typed validation outcomes instead of free-form strings.
- Downstream dependency behavior can consume the binding/validation result later.

### 4A-06: Legacy report/aggregation/chart adapter boundary
**Scope:** Backend / migration / reporting  
**Labels:** `phase:4-components`, `sprint:4A`, `type:refactor`, `status:backlog`, `area:backend`, `area:migration`, `area:reporting`
**Depends on:** `4A-01`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **Legacy report/aggregation/chart adapter boundary** in **Sprint 4A - Table Component Slice**. The primary goal is to keep legacy `Report`, `Aggregation`, and `Chart` behavior only as compatibility adapters where still required. It also includes work to prevent new core behavior from deepening those legacy concepts. This ticket spans **Backend / migration / reporting** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Keep legacy `Report`, `Aggregation`, and `Chart` behavior only as compatibility adapters where still required.
- Prevent new core behavior from deepening those legacy concepts.

**Acceptance criteria:**
- New table component behavior is implemented against component models, not legacy reporting nouns.
- Any legacy path still required is explicitly tagged as compatibility-only.
- At least one legacy-to-component adaptation path is documented or tested.

### 4A-07: Table component UAT
**Scope:** QA / components  
**Labels:** `phase:4-components`, `sprint:4A`, `type:test`, `status:backlog`, `area:components`
**Depends on:** `4A-01`, `4A-02`, `4A-03`, `4A-04`, `4A-05`, `4A-06`
**Milestone:** `Sprint 4A - Table Component Slice`


**Description:**
This issue implements **4A-07: Table component UAT** in **Sprint 4A - Table Component Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can create, version, publish, and view table components through the app.
- Component routes remain native SSR and clean in route checks.
- Legacy adapters do not appear as first-class authoring concepts in the UI.

---

## Sprint 4B: Chart And Stat Component Slice

### 4B-01: Visual component type system and validation
**Scope:** Backend / components  
**Labels:** `phase:4-components`, `sprint:4B`, `type:feature`, `status:backlog`, `area:backend`, `area:components`, `area:reporting`
**Depends on:** `4A-07`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **Visual component type system and validation** in **Sprint 4B - Chart And Stat Component Slice**. The primary goal is to extend the component model to support `Bar`, `Line`, `Pie/Donut`, and `StatCard` with typed validation. This ticket spans **Backend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Extend the component model to support `Bar`, `Line`, `Pie/Donut`, and `StatCard` with typed validation.

**Acceptance criteria:**
- Visual components are represented as component versions, not legacy chart entities.
- Validation distinguishes component-specific configuration problems predictably.
- Typed validation outcomes are consumable by UI and tests.

### 4B-02: `Bar` and `Line` component authoring
**Scope:** Frontend + backend / components  
**Labels:** `phase:4-components`, `sprint:4B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:components`
**Depends on:** `4B-01`, `4A-07`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **`Bar` and `Line` component authoring** in **Sprint 4B - Chart And Stat Component Slice**. The primary goal is to deliver authoring flows for `Bar` and `Line` components. This ticket spans **Frontend + backend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver authoring flows for `Bar` and `Line` components.

**Acceptance criteria:**
- Authors can create and edit `Bar` and `Line` components against valid dataset revisions.
- Preview or publish validation surfaces configuration errors clearly.
- Publishing succeeds only when required configuration is present.

### 4B-03: `Pie/Donut` and `StatCard` component authoring
**Scope:** Frontend + backend / components  
**Labels:** `phase:4-components`, `sprint:4B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:components`
**Depends on:** `4B-01`, `4A-07`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **`Pie/Donut` and `StatCard` component authoring** in **Sprint 4B - Chart And Stat Component Slice**. The primary goal is to deliver authoring flows for `Pie/Donut` and `StatCard` components. This ticket spans **Frontend + backend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver authoring flows for `Pie/Donut` and `StatCard` components.

**Acceptance criteria:**
- Authors can create and edit `Pie/Donut` and `StatCard` components through the app.
- Component-specific validation behaves consistently with the rest of the component family.
- The authoring UI does not fall back to old chart-specific workbench patterns.

### 4B-04: Visual component viewers
**Scope:** Frontend / components  
**Labels:** `phase:4-components`, `sprint:4B`, `type:feature`, `status:backlog`, `area:frontend`, `area:components`, `area:reporting`
**Depends on:** `4B-02`, `4B-03`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **Visual component viewers** in **Sprint 4B - Chart And Stat Component Slice**. The primary goal is to build application viewers for published visual components. This ticket spans **Frontend / components** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Build application viewers for published visual components.

**Acceptance criteria:**
- Published visual components render correctly in the app.
- Viewer routes reload and deep-link cleanly.
- Viewer failures identify binding/validation/render problems clearly.

### 4B-05: Legacy chart compatibility-only trim
**Scope:** Backend / migration / reporting  
**Labels:** `phase:4-components`, `sprint:4B`, `type:refactor`, `status:backlog`, `area:backend`, `area:migration`, `area:reporting`
**Depends on:** `4A-06`, `4B-01`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **Legacy chart compatibility-only trim** in **Sprint 4B - Chart And Stat Component Slice**. The primary goal is to continue reducing legacy chart ownership to compatibility-only behavior. This ticket spans **Backend / migration / reporting** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Continue reducing legacy chart ownership to compatibility-only behavior.

**Acceptance criteria:**
- No new visual-component behavior is implemented in legacy chart modules.
- Remaining legacy chart code is explicitly compatibility-only and tracked.
- At least one obsolete chart-specific product-facing path is removed or hidden behind the new component model.

### 4B-06: Visual component UAT
**Scope:** QA / components  
**Labels:** `phase:4-components`, `sprint:4B`, `type:test`, `status:backlog`, `area:components`, `area:reporting`
**Depends on:** `4B-01`, `4B-02`, `4B-03`, `4B-04`, `4B-05`
**Milestone:** `Sprint 4B - Chart And Stat Component Slice`


**Description:**
This issue implements **4B-06: Visual component UAT** in **Sprint 4B - Chart And Stat Component Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can build and view visual components without old chart/aggregation-specific workbench flows.
- Visual component routes remain native SSR and clean in route checks.
- Validation and render failures are attributable to the correct component type.

---

# Phase 5: Dashboards And Dependency Upgrade UX

## Sprint 5A: Dashboard Composition Slice

### 5A-01: Dashboard bounded-context backend structure
**Scope:** Backend / dashboards / architecture  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:refactor`, `status:backlog`, `area:backend`, `area:dashboards`
**Depends on:** `4B-06`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **Dashboard bounded-context backend structure** in **Sprint 5A - Dashboard Composition Slice**. The primary goal is to keep dashboard logic behind clear service/repo/DTO boundaries as it grows. This ticket spans **Backend / dashboards / architecture** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Keep dashboard logic behind clear service/repo/DTO boundaries as it grows.

**Acceptance criteria:**
- New dashboard composition behavior is not concentrated in a single god-file.
- Dashboard handlers are thin and defer orchestration to services.
- Repositories own persistence for touched dashboard flows.

### 5A-02: Dashboard directory/detail/create/edit/view SSR surfaces
**Scope:** Frontend / dashboards  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:feature`, `status:backlog`, `area:frontend`, `area:dashboards`
**Depends on:** `5A-01`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **Dashboard directory/detail/create/edit/view SSR surfaces** in **Sprint 5A - Dashboard Composition Slice**. The primary goal is to deliver application-grade directory, detail, create, edit, and view flows for dashboards. This ticket spans **Frontend / dashboards** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Deliver application-grade directory, detail, create, edit, and view flows for dashboards.

**Acceptance criteria:**
- A tester can create, inspect, edit, and view dashboards through SSR-owned routes.
- Dashboard routes deep-link and reload cleanly.
- No product-facing dashboard route revives bridge ownership.

### 5A-03: Component palette, placement, and composition
**Scope:** Frontend + backend / dashboards  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:components`
**Depends on:** `5A-01`, `5A-02`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **Component palette, placement, and composition** in **Sprint 5A - Dashboard Composition Slice**. The primary goal is to provide dashboard composition with component placement and layout management. This ticket spans **Frontend + backend / dashboards** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Provide dashboard composition with component placement and layout management.

**Acceptance criteria:**
- Authors can add and arrange component versions on a dashboard.
- Placement changes persist through normal application flows.
- The UI clearly distinguishes editable composition from viewer mode.

### 5A-04: Product-facing dashboard viewer polish
**Scope:** Frontend / dashboards  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:feature`, `status:backlog`, `area:frontend`, `area:dashboards`
**Depends on:** `5A-02`, `5A-03`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **Product-facing dashboard viewer polish** in **Sprint 5A - Dashboard Composition Slice**. The primary goal is to ensure dashboard viewers are readable and coherent for end users. This ticket spans **Frontend / dashboards** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Ensure dashboard viewers are readable and coherent for end users.

**Acceptance criteria:**
- Product-facing dashboard views emphasize reading and interpretation rather than authoring controls.
- Dashboard viewers handle empty, loading, and error states gracefully.
- Viewer routes remain hydration-clean and console-clean.

### 5A-05: Composition on `ComponentVersion` only
**Scope:** Backend / domain integrity  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:feature`, `status:backlog`, `area:backend`, `area:dashboards`
**Depends on:** `5A-01`, `4A-01`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **Composition on `ComponentVersion` only** in **Sprint 5A - Dashboard Composition Slice**. The primary goal is to enforce that dashboard composition depends on `ComponentVersion`, not legacy report/chart nouns. This ticket spans **Backend / domain integrity** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Enforce that dashboard composition depends on `ComponentVersion`, not legacy report/chart nouns.

**Acceptance criteria:**
- New dashboard composition records point to component versions.
- Legacy report/chart identifiers are not required to compose a new dashboard.
- Tests fail if a new dashboard path attempts to bypass the component model.

### 5A-06: Dashboard composition UAT
**Scope:** QA / dashboards  
**Labels:** `phase:5-dashboards`, `sprint:5A`, `type:test`, `status:backlog`, `area:dashboards`
**Depends on:** `5A-01`, `5A-02`, `5A-03`, `5A-04`, `5A-05`
**Milestone:** `Sprint 5A - Dashboard Composition Slice`


**Description:**
This issue implements **5A-06: Dashboard composition UAT** in **Sprint 5A - Dashboard Composition Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can assemble and view dashboards through the app.
- Dashboard routes remain native SSR and clean in route checks.
- Composition failures clearly identify placement, component binding, or persistence issues.

---

## Sprint 5B: Upgrade And Stale Dependency Slice

### 5B-01: Typed dependency health model
**Scope:** Backend / dependency management  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`
**Depends on:** `5A-06`, `3B-06`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **Typed dependency health model** in **Sprint 5B - Upgrade And Stale Dependency Slice**. The primary goal is to represent dependency health, warnings, and blockers using typed application models. This ticket spans **Backend / dependency management** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Represent dependency health, warnings, and blockers using typed application models.

**Acceptance criteria:**
- Dependency health is available as typed data rather than ad hoc string payloads.
- Downstream consumers can tell warning vs blocking states reliably.
- Typed results are reusable by publication guards and UI flows.

### 5B-02: Warning and blocking findings UI
**Scope:** Frontend / dependency UX  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:feature`, `status:backlog`, `area:frontend`, `area:dependencies`
**Depends on:** `5B-01`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **Warning and blocking findings UI** in **Sprint 5B - Upgrade And Stale Dependency Slice**. The primary goal is to build application screens or panels that show dependency health findings. This ticket spans **Frontend / dependency UX** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Build application screens or panels that show dependency health findings.

**Acceptance criteria:**
- A tester can inspect warning and blocking findings for dependent assets through the UI.
- Findings are grouped and labeled clearly enough to act on.
- The UI does not require reading raw IDs or logs to understand the problem.

### 5B-03: Carry-forward flows
**Scope:** Backend + frontend / upgrade UX  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:dependencies`
**Depends on:** `5B-01`, `5B-02`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **Carry-forward flows** in **Sprint 5B - Upgrade And Stale Dependency Slice**. The primary goal is to support compatible carry-forward flows for dependent assets. This ticket spans **Backend + frontend / upgrade UX** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Support compatible carry-forward flows for dependent assets.

**Acceptance criteria:**
- Compatible dependencies can be carried forward through the application.
- The UI clearly distinguishes automatic, recommended, and manual carry-forward outcomes if those modes exist.
- Carry-forward actions are recorded in a way the dependency health model can reflect.

### 5B-04: Rebinding flows for incompatible dependencies
**Scope:** Backend + frontend / upgrade UX  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:feature`, `status:backlog`, `area:frontend`, `area:backend`, `area:dependencies`
**Depends on:** `5B-01`, `5B-02`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **Rebinding flows for incompatible dependencies** in **Sprint 5B - Upgrade And Stale Dependency Slice**. The primary goal is to allow users to rebind incompatible assets to newer compatible upstream versions where allowed. This ticket spans **Backend + frontend / upgrade UX** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Allow users to rebind incompatible assets to newer compatible upstream versions where allowed.

**Acceptance criteria:**
- A tester can select and apply a rebind through the application.
- Rebinding validates against the same typed compatibility contracts introduced earlier.
- Failed rebind attempts return stable, actionable validation messages.

### 5B-05: Publication guards using typed compatibility/dependency outputs
**Scope:** Backend / publication rules  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:feature`, `status:backlog`, `area:backend`, `area:dependencies`
**Depends on:** `5B-01`, `3B-06`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **Publication guards using typed compatibility/dependency outputs** in **Sprint 5B - Upgrade And Stale Dependency Slice**. The primary goal is to block or warn publication based on typed compatibility/dependency findings. This ticket spans **Backend / publication rules** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Block or warn publication based on typed compatibility/dependency findings.

**Acceptance criteria:**
- Publishing a changed asset respects dependency blockers and warnings consistently.
- Guards consume typed findings from prior sprints rather than re-parsing ad hoc state.
- Tests cover at least one blocked publish and one warned-but-allowed publish path.

### 5B-06: Dependency upgrade UAT
**Scope:** QA / upgrade UX  
**Labels:** `phase:5-dashboards`, `sprint:5B`, `type:test`, `status:backlog`, `area:dependencies`
**Depends on:** `5B-01`, `5B-02`, `5B-03`, `5B-04`, `5B-05`
**Milestone:** `Sprint 5B - Upgrade And Stale Dependency Slice`


**Description:**
This issue implements **5B-06: Dependency upgrade UAT** in **Sprint 5B - Upgrade And Stale Dependency Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- A tester can update dependent assets and resolve or defer findings through the UI.
- Dependency health and upgrade routes remain native SSR and clean in route checks.
- Failures clearly identify warning display, carry-forward, rebinding, or publication-guard issues.

---

# Phase 6: Migration, Hardening, And Pilot Readiness

## Sprint 6A: Migration And Legacy Mapping Slice

### 6A-01: Mapping docs aligned to datasets, components, and dashboards
**Scope:** Documentation / migration  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:docs`, `status:backlog`, `area:datasets`, `area:components`, `area:dashboards`
**Depends on:** `5B-06`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **Mapping docs aligned to datasets, components, and dashboards** in **Sprint 6A - Migration And Legacy Mapping Slice**. The primary goal is to update mapping docs so migration guidance matches the new model. This ticket spans **Documentation / migration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Update mapping docs so migration guidance matches the new model.

**Acceptance criteria:**
- Mapping docs refer to datasets, component versions, and dashboards rather than legacy reporting nouns as the forward model.
- Operators can follow the docs without translating from old internal terminology manually.
- Docs are versioned or checked in near the code they describe.

### 6A-02: Operator verification paths aligned to canonical application routes
**Scope:** Frontend / operators / migration  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:feature`, `status:backlog`, `area:frontend`, `area:migration`, `area:operators`
**Depends on:** `5B-06`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **Operator verification paths aligned to canonical application routes** in **Sprint 6A - Migration And Legacy Mapping Slice**. The primary goal is to update migration/operator screens so they point to canonical native application routes when replacements exist. This ticket spans **Frontend / operators / migration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Update migration/operator screens so they point to canonical native application routes when replacements exist.

**Acceptance criteria:**
- Migration/operator flows link to the real product/detail screens for validation where possible.
- Operators do not need to bounce through obsolete builder/bridge pages to verify outcomes.
- Touched operator routes remain SSR-owned and clean in route checks.

### 6A-03: Verification flows for migrated assets
**Scope:** Full stack / migration  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:feature`, `status:backlog`, `area:migration`
**Depends on:** `6A-01`, `6A-02`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **Verification flows for migrated assets** in **Sprint 6A - Migration And Legacy Mapping Slice**. The primary goal is to provide application-grade verification flows for migrated datasets, components, and dashboards. This ticket spans **Full stack / migration** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Provide application-grade verification flows for migrated datasets, components, and dashboards.

**Acceptance criteria:**
- Operators can validate migration outcomes using the same application surfaces used for normal assets.
- Verification distinguishes imported-but-invalid outcomes from successful imports.
- Errors surface as stable operator-facing findings rather than raw traces.

### 6A-04: Inventory all remaining hybrid-shell and legacy-builder surfaces
**Scope:** Architecture / cleanup planning  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:refactor`, `status:backlog`, `area:frontend`, `area:migration`
**Depends on:** `6A-02`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **Inventory all remaining hybrid-shell and legacy-builder surfaces** in **Sprint 6A - Migration And Legacy Mapping Slice**. The primary goal is to create a complete inventory of remaining hybrid-shell and builder-era surfaces. This ticket spans **Architecture / cleanup planning** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Create a complete inventory of remaining hybrid-shell and builder-era surfaces.

**Acceptance criteria:**
- Every remaining `application.rs`, `/bridge/*`, or legacy builder dependency is listed with an owner.
- The inventory distinguishes primary routes from operator-only or compatibility-only surfaces.
- The list is updated enough to support Sprint 6B closeout.

### 6A-05: Deletion plan for each remaining legacy surface
**Scope:** Planning / cleanup  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:refactor`, `status:backlog`, `area:migration`
**Depends on:** `6A-04`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **Deletion plan for each remaining legacy surface** in **Sprint 6A - Migration And Legacy Mapping Slice**. The primary goal is to for each remaining legacy surface, record the deletion path, replacement route, and blocker if any. This ticket spans **Planning / cleanup** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- For each remaining legacy surface, record the deletion path, replacement route, and blocker if any.

**Acceptance criteria:**
- Every inventoried legacy surface has a deletion plan or an explicit reason it remains out of scope.
- The plan is specific enough to turn into tickets.
- Primary application routes are prioritized ahead of rare operator-only surfaces.

### 6A-06: Migration/operator UAT
**Scope:** QA / operators / migration  
**Labels:** `phase:6-pilot-readiness`, `sprint:6A`, `type:test`, `status:backlog`, `area:migration`, `area:operators`
**Depends on:** `6A-01`, `6A-02`, `6A-03`, `6A-04`, `6A-05`
**Milestone:** `Sprint 6A - Migration And Legacy Mapping Slice`


**Description:**
This issue implements **6A-06: Migration/operator UAT** in **Sprint 6A - Migration And Legacy Mapping Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- Operators can validate migration outcomes using real application surfaces.
- Touched migration routes remain native SSR and clean in route checks.
- The inventory and deletion plan are current at sprint close.

---

## Sprint 6B: Pilot Hardening Slice

### 6B-01: End-to-end coverage matrix for primary application flows
**Scope:** QA / end-to-end  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:test`, `status:backlog`, `area:backend`
**Depends on:** `6A-06`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **End-to-end coverage matrix for primary application flows** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to build a coverage matrix for primary user and operator flows. This ticket spans **QA / end-to-end** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Build a coverage matrix for primary user and operator flows.

**Acceptance criteria:**
- End-to-end coverage exists for the primary signed-in application flows that matter to pilot testing.
- Coverage explicitly includes auth/session, core authoring, workflow/runtime, datasets/components/dashboards, and migration verification where appropriate.
- The coverage matrix is checked in and easy to audit.

### 6B-02: Smoke-path coverage and CI enforcement
**Scope:** QA / CI  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:test`, `status:backlog`, `area:ci`
**Depends on:** `6B-01`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Smoke-path coverage and CI enforcement** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to ensure smoke paths run automatically and gate regressions. This ticket spans **QA / CI** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Ensure smoke paths run automatically and gate regressions.

**Acceptance criteria:**
- Smoke paths run in CI or an equivalent enforced gate.
- Failures identify which major area regressed.
- Smoke coverage remains aligned with the primary pilot flows.

### 6B-03: Final permission hardening audit
**Scope:** Backend + QA / security / authorization  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:feature`, `status:backlog`, `area:backend`, `area:authorization`, `area:security`
**Depends on:** `6A-06`, `6B-01`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Final permission hardening audit** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to audit permission behavior across primary routes and actions. This ticket spans **Backend + QA / security / authorization** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Audit permission behavior across primary routes and actions.

**Acceptance criteria:**
- Primary routes and critical actions are verified against expected role/scope rules.
- Unauthorized access fails safely and predictably at both route and action level.
- Permission findings are resolved or explicitly documented before pilot close.

### 6B-04: Final session and error-envelope audit
**Scope:** Backend / security / API quality  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:feature`, `status:backlog`, `area:backend`, `area:auth`, `area:security`
**Depends on:** `6A-06`, `6B-01`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Final session and error-envelope audit** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to re-audit session handling and client-visible error behavior before pilot use. This ticket spans **Backend / security / API quality** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Re-audit session handling and client-visible error behavior before pilot use.

**Acceptance criteria:**
- No primary route depends on deprecated browser auth behavior.
- Client-visible payloads for primary flows use stable codes/messages.
- Server logs retain enough detail for diagnosis without leaking internal detail to end users.

### 6B-05: Performance cleanup and budgets
**Scope:** Performance / frontend + backend  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:refactor`, `status:backlog`, `area:frontend`, `area:backend`, `area:performance`
**Depends on:** `6B-01`, `6B-02`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Performance cleanup and budgets** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to resolve high-value performance issues found in pilot preparation. It also includes work to add or tighten budgets where the product now has enough shape to measure them. This ticket spans **Performance / frontend + backend** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Resolve high-value performance issues found in pilot preparation.
- Add or tighten budgets where the product now has enough shape to measure them.

**Acceptance criteria:**
- Known high-value bottlenecks on primary routes are improved or triaged explicitly.
- At least one measurable performance baseline or budget is checked for primary application routes.
- Performance work does not regress SSR ownership or route stability.

### 6B-06: Remove remaining primary-route hybrid shell dependencies
**Scope:** Full stack / migration / cleanup  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:refactor`, `status:backlog`, `area:frontend`, `area:migration`, `area:platform`
**Depends on:** `6A-05`, `6B-01`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Remove remaining primary-route hybrid shell dependencies** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to close the roadmap only when the hybrid shell is gone from primary application routes. This ticket spans **Full stack / migration / cleanup** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Close the roadmap only when the hybrid shell is gone from primary application routes.

**Acceptance criteria:**
- No remaining primary route depends on `application.rs`, `/bridge/*`, or retained legacy bridge assets.
- Any residual legacy surface is non-primary and explicitly tracked with rationale.
- Route ownership reports confirm the primary app is fully native SSR.

### 6B-07: Unsupported-v1 documentation
**Scope:** Documentation / product communication  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:docs`, `status:backlog`, `area:documentation`
**Depends on:** `6B-03`, `6B-04`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **Unsupported-v1 documentation** in **Sprint 6B - Pilot Hardening Slice**. The primary goal is to document what is intentionally out of scope or unsupported for v1. This ticket spans **Documentation / product communication** concerns and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Work:**
- Document what is intentionally out of scope or unsupported for v1.

**Acceptance criteria:**
- Unsupported behaviors are documented in one easy-to-find location.
- The docs use product language rather than internal shorthand.
- Pilot testers and operators can tell what is intentionally deferred.

### 6B-08: Final pilot-readiness UAT closeout
**Scope:** QA / release readiness  
**Labels:** `phase:6-pilot-readiness`, `sprint:6B`, `type:test`, `status:backlog`, `area:backend`
**Depends on:** `6B-01`, `6B-02`, `6B-03`, `6B-04`, `6B-05`, `6B-06`, `6B-07`
**Milestone:** `Sprint 6B - Pilot Hardening Slice`


**Description:**
This issue implements **6B-08: Final pilot-readiness UAT closeout** in **Sprint 6B - Pilot Hardening Slice** and should leave the touched surface in a shippable state that satisfies the acceptance criteria below.

**Acceptance criteria:**
- The application remains fully testable through intended UI flows after hardening.
- Primary routes are native SSR, hydration-clean, and console-clean.
- Pilot closeout includes explicit verification that the roadmap end-state for hybrid-shell removal is met.

---

# Suggested execution notes

- **Treat complete sprints as locked.** Only take their regression tickets when a new sprint touches those areas or when adding coverage is the fastest way to protect later changes.
- **If you want issue labels**, a practical starter set is: `area:frontend`, `area:backend`, `area:domain`, `area:qa`, `type:refactor`, `type:feature`, `type:hardening`, `type:docs`, `priority:p0`, `priority:p1`.
- **Good Codex task size:** one ticket, or one sub-slice of a ticket if the ticket spans both backend and frontend and you want separate PRs.