# Tessara Roadmap

This roadmap is authoritative as of July 15, 2026. It starts from the completed Sprint 6A control-plane baseline and sequences the next work around Tessara's modular application-platform direction: Core plus selected, separately deployed full-stack modules, composed into independently supported applications through versioned contracts and declarative blueprints.

## Delivery Rule

Every future sprint is a full vertical slice.

- Every sprint must deliver both underlying functionality and usable application UI.
- The application must remain in a user-testable condition in the intended end-user-facing shape after each sprint.
- Backend-only completion does not satisfy roadmap completion.
- Internal/admin/configuration screens may evolve inside the same sprint, but they do not replace the requirement for coherent application UI.
- Existing route and UI surfaces must stay on the native Leptos SSR platform.
- Touched surfaces do not count as complete if they reintroduce HTML-string route shells, `inner_html` route injection, `/bridge/*` assets, or JavaScript controller ownership for application UI.
- The hybrid shell is gone from active routing; future work must preserve that baseline.

## Sprint completion protocol (applies to every sprint)

- Run a local deployment refresh with `.\scripts\local-launch.ps1` for standard
  updates, or `.\scripts\local-launch.ps1 -FreshData` only when the current
  sprint's ordered protocol has reached its destructive fresh-install step.
- Sprint 6A closeout is governed by the plan's
  [ordered required Gates 1–6](./sprints/sprint-6a-plan.md#ordered-required-gates),
  not by a single fresh-deployment command or this roadmap summary. Execute the
  gates in order: source/contract checks, three-database integration,
  populated-upgrade plus compatibility/restore proof, closing-build regression
  on the restored Sprint 5A demo target after the closing startup upgrades it
  with seeding disabled, fresh installation, and final cleanliness. The
  representative populated-upgrade fixture remains a separate compatibility
  proof database and is not the Gate 4 browser candidate.
- Sprint 6A must retain two non-interchangeable acceptance sets. Gate 4 produces
  `deployment-upgraded.json` plus upgraded smoke, UAT, exact-manifest Playwright,
  and nondisclosure evidence before any fresh reset. Gate 5 then produces a
  separate `deployment-fresh.json` plus fresh smoke, UAT, exact-manifest
  Playwright, and nondisclosure evidence. Gate 3's rollback package and verified
  backup/restore evidence are also mandatory; neither state-specific set
  substitutes for them.
- Print and run the sprint UAT, smoke, Playwright, and any sprint-specific
  release/conformance commands with the exact deployment-evidence path and data
  state required by that sprint's ordered plan. For Sprint 6A, use the commands
  and state-specific output paths in Gates 4 and 5 without filters or a
  development-mode bypass.
- Confirm the UAT script output includes current route ownership and role-gated behavior before closing the sprint.
- Confirm any sprint that adds or changes permission-controlled behavior updates `docs/playwright-permissions-scenarios.md` and includes positive and negative Playwright coverage where currently executable.
- Confirm every route surface touched in the sprint remains under native SSR ownership before closing the sprint.
- Confirm route ownership, hydration, and browser-console cleanliness for every touched route before closing the sprint.
- For any sprint that adds or changes a module boundary, validate the module manifest, contract compatibility, same-origin routing, scope-bound grants and downstream-audience authorization exchange, database isolation, module outage behavior, and platform conformance suite. A pre-runtime contract sprint must name each check that is not yet executable, run the strongest contract-level substitute, carry the real runtime check into the first runtime sprint as an entry criterion, and never report a deferred check as passed.
- Confirm application-composition changes produce a valid Blueprint, resolved lockfile, deterministic plan, and read-back result once those platform capabilities exist.
- If a detour sprint lands outside the numbered roadmap, reconcile this file with the codebase before selecting the next roadmap sprint.

## Cross-Cutting Delivery Constraints

- No new user-facing behavior may be added through HTML-string route shells, `inner_html` injection, `/bridge/*`, or retained legacy bridge assets.
- Any sprint that touches `auth`, `hierarchy`, `forms`, `workflows`, `submissions`, or `reporting` must move touched backend behavior toward bounded-context structure with explicit `router`, `handlers`, `service`, `repo`, and `dto` boundaries rather than expanding large vertical files.
- Browser authentication for native UI routes must use a server-managed session contract. JavaScript-managed bearer tokens may remain only for explicit CLI, script, or testing flows.
- Client-visible error payloads must use stable application codes and messages. Raw database and internal error strings must not be exposed to end users.
- Any sprint that exposes scoped analytical, workflow, response, dataset, component, chart, or dashboard data must prove operator scope filtering with negative regression coverage before closeout.
- New cross-feature relationships must move toward versioned functional contracts and typed resource references. Do not introduce new cross-module table access, database credentials, foreign keys, or stored deployment URLs.
- Core remains authoritative for Organization, identity and user management, RBAC, the shared shell, and module composition. Module-specific domain rules, persistence, versioning, and lifecycle policy belong to the owning module.
- Dependency-audit failures are treated as release blockers unless the advisory is documented as unreachable, accepted, and tied to a replacement or removal path.
- Every sprint close must verify route ownership, hydration, and browser-console cleanliness for touched routes in addition to the existing UAT script.

## Current State Of Development

### Implemented baseline

The codebase already includes a substantial vertical foundation:

- local Docker-based development, runnable Rust service, seeded demo workflows, and smoke helpers
- explicit login flow, session handling, capability/scope metadata, role-aware navigation, and route guards
- application-grade administration screens for users, roles, scoped access, and delegations
- admin-managed role creation and assignment with capability bundles
- configurable organization hierarchy and metadata-backed nodes
- form, form version, section, and field support with publish lifecycle
- draft/save/submit response flows and review behavior
- reporting/storage slices for datasets, reports, aggregations, charts, and dashboards
- legacy fixture validation, dry-run, import rehearsal, and demo seed paths
- a Leptos SSR shell with root-level native product routes for Home, Organization, Forms, Workflows, Responses, Components, Datasets, Dashboards, and Administration; the former Migration surface is retired and has no live route
- extracted Leptos feature crates for shared UI, Datasets, Forms, Workflows, Responses, and Organization, with root-owned route adapters preserving shell, auth, route parsing, hydration, CSS, and cargo-leptos ownership
- Sprint 2B authentication hardening: Argon2id credential storage, server-side session expiry/revocation/last-seen tracking, same-origin `HttpOnly` browser cookies, stable auth/session errors, and native SSR login/session behavior
- UI Overhaul 2.0 detour work: approved shell navigation posture, access-denied redirect plus transient feedback, sidebar footer account/scope/theme context, queue-first home posture, explorer-oriented organization work, section-oriented form-builder UI, and section description/column-count persistence
- Sprint 4A Dataset Catalog and Thin Table Components: searchable Dataset tags, Dataset provenance lineage, one thin Table Component over Dataset major-line outputs with last-mile projection/filter defaults, edit-screen component versioning/publishing, and shared interactive table rendering for Dataset previews and Component viewers
- Sprint 5A Dashboard Composition: native directory/detail/editor/viewer flows over stable `ComponentVersion` identities, revisioned placement composition, bounded execution, and scoped redaction
- Sprint 6A Module Contract and Core Control Plane: one stable Application Installation and Core runtime observation; exact Manifest, Feature Declaration, contract, semantic-destination, typed-reference, and real Release/Instance public types; seven versioned transition descriptors with six active in-process contributions and retired Migration; Core-owned module inventory/provenance APIs and native Module Management UI; and revisioned band-restricted navigation policy, without Release/Instance persistence, installation, execution, or mutation

### Closed Sprint 6A UI baseline (input to Sprint 6A-UI)

Until Sprint 6A-UI lands, the application shell exposes these meaningful user-testable surfaces. The Sprint 6A-UI block below explicitly supersedes the fixed group, anchor, band, and Administration-landing details while preserving unrelated route behavior:

- role-aware login and shared home entry
- product-area navigation for Home, Organization, Forms, Workflows, Responses, Components, Dashboards, Datasets, and Administration; Migration is retained only as retired historical/support inventory
- a fixed Module Management item in the `Admin` group: effective installation-global `modules:read` exposes the inventory and policy readback, `modules:manage_navigation` implies read and enables policy controls, and the separate Administration item remains `admin:all`-only
- dedicated list/detail/create/edit flows for major top-level entities
- dedicated administration list/detail/create/edit/access flows for users and roles
- visible separation between product-facing and internal/operator areas
- Components and Datasets are native internal inspection surfaces
- Report, aggregation, and chart APIs remain compatibility implementation details, but reports are no longer mounted as a primary UI route or forward planning model

### Current implementation gaps

The contract and control-plane foundation is complete, but Tessara does not yet materialize or operate a real Module Release or Module Instance. Sprint 6A-UI first replaces Sprint 6A's rigid navigation-band policy with configuration-driven groups and harmonizes the related administration UI without changing that platform boundary. Sprint 6B then must:

- persist and mutate real Module Releases and Instances without treating Sprint 6A transition descriptors as deployable providers
- introduce a Supervisor-rooted `tessara-oci-v1` runtime, same-origin gateway, verifiable installation context, health/readiness observation, and controlled lifecycle operations
- add scope-bound grants or exact Core authorization decisions with downstream audience, freshness, replay, and revocation proof
- establish one database per module instance in the installation's PostgreSQL cluster, with dedicated credentials and no cross-module database access
- install the Sprint 6B reference module as the first independently deployed full-stack process and database
- define deterministic Application Blueprint, lockfile, validate, plan/diff, apply, and read-back operations for human and LLM composition
- prove scoped authorization, lifecycle observation, unavailable-state behavior, and diagnostics across actual module boundaries
- move remaining non-Core feature areas into independently deployed and supported modules before broad pilot hardening

### Frontend transition baseline

The completed web refactoring pass remains useful because it created explicit feature seams without changing route behavior. Those crates are now transitional aids toward full-stack module ownership rather than the final architecture.

- Keep current root route adapters, shell, auth/session policy, hydration entrypoint, CSS, and assets stable until the same-origin gateway and module SDK can replace those responsibilities deliberately.
- Avoid new dependencies from feature crates into root application policy or sibling feature internals.
- Treat each non-Core feature area as a candidate module boundary owning UI, API, persistence, configuration, diagnostics, and contracts together.
- Promote stable wire schemas into module-owned contract crates or generated clients; do not use shared DTO code as a shortcut to shared domain ownership.
- Build the shared design-system and module SDK around SSR-compatible shell integration, authenticated context, semantic navigation, typed references, stable errors, health, and conformance testing.
- Use full-page same-origin module routes as the default. Runtime-loaded remote UI code, iframes, and browser-side microfrontend composition are not required.
- Continue measuring release artifacts and route behavior, but evaluate extraction based on the complete module boundary rather than frontend compile time alone.

## Current Transitional Architecture

The current implementation still contains a transitional reporting stack:

```text
Forms -> Dataset -> Report -> Aggregation -> Chart -> Dashboard
```

That stack reflects shipped code and historical progress, but it is not the forward target architecture for planning.

What remains useful from the transitional stack:

- dataset execution and multi-source composition work
- chart/dashboard viewing patterns
- migration rehearsal and reporting verification infrastructure

What must change:

- target planning should stop assuming `Report`, `Aggregation`, and `Chart` remain separate future-state asset families
- future UI and architecture work should converge on `Component` as the presentation asset

## Target Architecture

The platform topology includes an out-of-band control/materialization plane plus the user request/data path:

```text
Installation Supervisor + authoritative ledger
  `-- materializes Core Release, gateway component, module instances, and database bindings

Browser -> same-origin gateway and Core shell
  |-- Core (Organization, identity/users, RBAC, module control plane) -> Core database
  `-- selected full-stack module applications -----------------------> one database each
```

Modules advertise versioned functional contracts, resource types, configuration, semantic destinations, navigation contributions, security capabilities, and health. Core validates and composes those contributions but does not own module product semantics or data. Cross-module integration uses APIs, events, exports, and durable typed resource references; direct cross-module database access is prohibited.

The current first-party capability flow remains the reference application:

```text
Forms/Workflows -> Responses -> Materialized Sources -> Dataset major-line contract -> ComponentVersion -> Dashboard
```

This is a product flow, not a deployment diagram. Forms, Workflows, Responses, Datasets, Components, and Dashboards become separately deployed full-stack modules. Module owners decide mutation, versioning, publication, lifecycle, audit, and historical-review rules. The platform guarantees reference owner/type stability and declared state observation, not universal immutability.

Application construction converges on machine-readable module catalogs and a deterministic `Blueprint -> validate -> resolve -> plan/diff -> lockfile + Materialization Plan`, followed by a separate approval envelope and `Supervisor apply -> verify` workflow suitable for administrators, deployment automation, and LLM clients without conflating planning and approval authority.

## Durable Carry-Forward Backlog

The items below preserve still-valid future work from completed sprint plans, handoff notes, UAT findings, and review artifacts. They are not active sprint scope unless a later roadmap slice explicitly pulls them forward. Core-owned work stays here; module-specific product work should move into the owning module roadmap as those repositories or release units are established; composition-wide behavior stays in the reference-application roadmap.

Access and administrative feature areas:

- Add an in-app `/administration/users/new` flow so admins can create users without direct API calls, including email, display name, password, active status, initial roles, and a clear follow-up path to scope/delegation access assignment.
- Replace the current broad Administration grouping with clearer Core areas for User Management, Roles and Access, Organization Schema, and Module Management. Module-provided administration remains owned by the contributing module and is reached through Core's module inventory or its advertised administrative destination.
- Add direct user capability-assignment affordances only after a role-template and capability-drift model is explicit enough that administrators can understand deviations from role bundles.
- Add a dedicated administrative workflow assignment detail route when assignment operations grow beyond filtered list management. Future scope should include reassignment, admin completion, deactivation/reactivation, mutation authorization, and capability decisions.

Workflow and response runtime:

- Improve workflow assignment lists with table-grade sort and filter controls for workflow, node, assignee, assignment status, and acting context.
- Decide whether workflow assignments need explicit one-time versus recurring behavior before introducing recurring assignment UX or scheduling semantics.
- Review workflow publish semantics for branching and sibling step form scopes. Older workflow-runtime expectations treated some branching/sibling scope combinations as invalid at publish time, while the current workflow publisher permits them. Decide whether this was stale test coverage or a dropped product rule; then either implement and document publish-time validation with regression coverage, or document the permissive behavior and keep tests aligned with it.
- Extend workflow runtime beyond same-assignee automatic handoff only when there is a complete model for per-step assignees, operator-mediated handoff, and capability-aware reassignment.
- In the Forms and Workflows module roadmaps, decide their respective draft, active, retired, and publication rules. Expose those states through typed contracts so Responses can retain its references and Workflows can decide whether assignments migrate; do not make Core the owner of that policy.
- Keep response starts assignment-only. Form-first starts should continue to flow through generated single-form workflow shortcuts and then start a workflow assignment.
- Make workflow steps the owner of target/context semantics, including explicit workflow availability nodes, step target metadata, cross-step data passing, prefills, hidden or locked carried-forward values, derived target nodes, and future nonlinear branching.
- Redesign Home delegated-work discovery so accounts with accessible delegate work can discover, switch, or default into delegated work without relying on the Responses route first.

Datasets, components, dashboards, and operations:

- Before the next substantial dataset compiler or materialization feature, split `crates/tessara-api/src/datasets/mod.rs` mechanically into focused handler, access, repository, materialization, and compiler modules.
- Sprint 3C review adoption intentionally defers that `datasets/mod.rs` split to a mechanical follow-up branch so blocker fixes stay focused on publish/materialization semantics.
- After the dataset module split, or when another internal pipeline column is added beyond `__row_id` and `__restriction_tier`, introduce a small pipeline schema abstraction that separates internal CTE columns from user-visible dataset fields.
- Split revision field loading from `DatasetSummary` if `/api/datasets` payload size, latency, or call-site needs show that output fields and revision field summaries are too heavy by default.
- Continue the ordered dataset operation-pipeline direction: projection, aggregation, calculated fields, filters, and view restrictions should be composable in saved operation-list order, including multiple operation instances where useful.
- Keep legacy reporting endpoints adapter-only while the reference model moves through `Dataset major-line contract -> ComponentVersion -> Dashboard`; dashboard composition should depend on component versions rather than legacy report/chart nouns.
- Treat `/operations` and `operations:view` as read-only status visibility. Keep it separate from `analytics:refresh`, refresh/admin mutations, row-level analytical data, and report execution details unless a later sprint explicitly defines scoped mutation capabilities.
- Add a refresh ledger or job-history model only if Operations grows beyond derived readiness/status.

UI and migration:

- Evaluate a focused migration of the shared placement drag/resize interactions from owner-bound mouse listeners to Pointer Events with `setPointerCapture` so mouse, touch, and pen input share one event path. Keep the current RAII cleanup model until that work has dedicated coverage for `pointercancel`/`lostpointercapture`, keyboard and focus accessibility, nested modal/side-sheet interactions, and mobile/pen regressions.
- Preserve long response values in readable table/detail presentations with wrapping, truncation, or drill-in behavior before expanding dense review surfaces.
- Review RBAC-heavy tables and details as Administration grows, especially capability bundle display, scope/delegation density, and capability-drift affordances.
- Keep migration/operator verification pointed at canonical application routes and remove legacy adapter surfaces once their replacement route, validation, and rollback path are explicit.

## Frontend Platform Foundation

This section records the completed foundation sequence that led to the current native UI baseline. It is historical planning context, not permission to reintroduce bridge-backed surfaces.

### Platform Sprint A: Cargo-Leptos Foundation

**Outcome:** the UI runs through a real `cargo-leptos` build pipeline while keeping the current single-binary deployment shape.

**Build:**

- multi-package `cargo-leptos` workspace metadata with `tessara-api` as the server binary and `tessara-web` as the frontend library
- built wasm/js package served by the existing `axum` binary
- shared stylesheet emitted through the `cargo-leptos` pipeline
- hydrated Leptos router preserving the current route surface
- cargo-leptos assets isolated from Rust string literals and served through the API binary

**Application UI delivered this sprint:**

- preserved existing routes remain user-testable
- the app shell and current route bodies still render while the runtime/build contract moves under them

**Historical bridge status after this sprint:** bridge-backed surfaces still existed at that point; they have since been removed from active routing.

### Platform Sprint B: Route Parity With Isolated Bridge

**Outcome:** preserved routes run through the Leptos runtime contract, and every remaining bridged surface has a named replacement target.

**Build:**

- route-by-route mapping of preserved URLs to Leptos-owned route components
- body-level route metadata controlled by the Leptos shell/runtime
- feature-local transport boundaries for UI/API interaction
- route inventory documenting which surfaces still rely on the retained JavaScript bridge

**Application UI delivered this sprint:**

- preserved routes continue to work without URL churn
- the bridge is explicit, isolated, and no longer spread as the default frontend architecture

**Bridge surfaces still expected after this sprint:**

- workflow-heavy product and internal pages that have not yet reached native Leptos parity

### Platform Sprint C: Split Heavy Routes And Start Bridge Removal

**Outcome:** route/widget splitting is active for heavy operator flows, and the first preserved routes stop depending on the legacy bridge.

**Build:**

- route-level code splitting for heavy operator routes; the historical `/migration` candidate is retired and is no longer a live splitting target
- bundle-loading verification in end-to-end coverage
- removal of the bridge from the first product/internal surfaces that have native replacements
- browser-console and hydration-error enforcement in end-to-end tests

**Application UI delivered this sprint:**

- the shared shell stays light
- heavy routes load additional client code only when entered
- at least one preserved product route and one internal/operator route no longer require the bridge

## Phase 1: Identity, Access, Organization, And Form Authoring

### Sprint 1A: User Management And Authentication (Complete)

**Outcome:** administrators manage users through application UI, and users authenticate into the intended shell.

**Build:**

- user directory/detail/create/edit flows
- login/session handling refinement
- explicit error feedback for failed login attempts
- account status handling and current-user visibility
- stable home-entry behavior after authentication

**Application UI delivered this sprint:**

- usable user-management screens in internal/admin surfaces
- stable login and post-login home entry in the application shell

**User-testable exit condition:** a tester can sign in, browse users, create or edit a user, and reach the correct application shell without direct DB or API work.

### Sprint 1B: RBAC And Scoped Role Assignment (Complete)

**Outcome:** roles and scoped assignments are manageable through application UI and visibly affect product/internal behavior.

**Build:**

- role catalog and capability-bundle management
- scoped role-assignment flows
- descendant-scope behavior
- route/action gating tied to assignments
- accessible data-grid administration views for capability bundles and scope assignments so larger role/scope sets remain readable and editable

**Application UI delivered this sprint:**

- role list/detail/edit screens
- role-assignment screens
- visible role-aware navigation and action gating

**User-testable exit condition:** a tester can assign roles and scopes in the UI and verify that navigation, actions, and visible surfaces change correctly.

### Sprint 1C: Organization Management (Complete)

**Outcome:** organization hierarchy browsing and editing work through the application shell.

**Build:**

- hierarchy traversal and calmer detail presentation
- node detail, create, and edit flows
- scoped terminology support
- scope-aware naming so top-level organization destinations reflect the highest assigned node type such as `Partner List`
- full-width hierarchy navigation to replace flat card-only browsing for organization traversal
- contextual internal configuration touchpoints where needed

**Application UI delivered this sprint:**

- end-user-facing organization directory/detail/create/edit flows
- scope-aware list titles and hierarchy navigation that make assigned subtrees understandable at a glance
- internal configuration touchpoints that do not dominate the product surface

**User-testable exit condition:** a tester can browse and manage organization nodes without IDs or workbench-only flows.

### Sprint 1D: Forms, Fields, And Version Authoring (Complete)

**Outcome:** form authoring is application-grade and explicitly supports field creation and editing.

**Build:**

- form directory/detail/create/edit flows
- form version lifecycle visibility
- field creation, editing, deletion, and reordering
- option sets and lookup-source authoring touchpoints
- workflow-attachment points for published form versions

**Application UI delivered this sprint:**

- dedicated form builder/editor screens inside the app
- field-authoring screens and controls that no longer depend on builder-only fallback flows

**User-testable exit condition:** a tester can create a form, add/edit/remove/reorder fields, publish a version, and inspect status entirely through UI.

### Sprint 1E: Form Semantic Versioning And Compatibility Automation (Deferred To Forms Module Roadmap)

**Outcome:** the Forms module defines and automates its own versioning and compatibility policy without asking users for manual labels or exposing internal compatibility identifiers.

**Build:**

- publish-time server-side semantic version derivation for form versions
- structural compatibility classification at publish time
- automatic major-version reuse for compatible revisions and automatic major-version rollover for breaking revisions
- publish-time diff summary that explains whether the revision is `PATCH`, `MINOR`, or `MAJOR`
- typed `FormVersion` resolution and state-observation contracts for Workflow, Response, and other authorized consumers
- provider-owned diff, compatibility, and lifecycle metadata exposed through the Forms contract without direct consumer access to Forms storage

**Application UI delivered this sprint:**

- draft version flows that defer semantic version and major-version assignment until publish
- publish review screens that show the proposed semantic version, Forms-owned compatibility classification, and lifecycle effect before confirmation
- compatibility status messaging on form detail and edit routes so authors can see when a published revision stayed in the current major line or started a new one

**User-testable exit condition:** a tester can revise and publish a FormVersion and receive the Forms module's declared versioning, compatibility, and lifecycle outcome without entering version labels or interpreting internal identifiers. Consumer observation and reaction are delivered in Sprint 7B and the relevant extraction sprint.

### Sprint 1F: Application UI Guidance Alignment (Complete)

**Outcome:** the current application UI aligns with the canonical shell, page-family, and responsive guidance before deeper workflow-runtime delivery continues.

**Build:**

- shared shell alignment to `ui-guidance.md` for top app bar, sidebar behavior, page headers, breadcrumbs, spacing, responsive layout, theme controls, and internal-area distinction
- route-by-route UI cleanup for existing `Home`, `Organization`, `Forms`, `Responses`, `Dashboards`, `Administration`, and `Migration` surfaces
- organization browse and detail polish toward the hierarchy-first, scope-aware direction already called out in canonical docs
- reduction of builder-era and transitional framing in end-user-facing application surfaces without adding new backend workflow scope

**Application UI delivered this sprint:**

- coherent shared shell with utility-only top bar and visible static global search
- consistent directory, detail, and editor framing across the existing core routes
- clearer product-vs-internal separation, with Administration subtle and Migration subordinate to the main application shell

**User-testable exit condition:** a tester can sign in and move through the existing application routes in a coherent shell on desktop and narrow widths, without builder-centric framing, shell-level horizontal scroll, hydration regressions, or browser-console errors.

### Sprint 1G: Tessara UI Component System Foundation (Complete)

**Outcome:** shared application surfaces move onto a predictable internal component layer so future route work stops depending on ad hoc page-local markup and styling.

**Build:**

- establish shared UI primitives as the internal component layer for Tessara
- use the consolidated `ui-guidance.md` as the specification source for component appearance and behavior
- extract the first stable primitives for shared page headers, action groups, cards, panels, metadata strips, inputs, field wrappers, and table or list toolbar patterns
- move touched route surfaces onto shared primitives incrementally while keeping the shared shell stable and SSR-first
- stop adding new bespoke route-level UI patterns when an approved component spec already exists

**Application UI delivered this sprint:**

- current shared routes begin rendering through a recognizable shared visual system instead of route-by-route markup drift
- new Sprint 2A assignment and response-start work can land on top of shared component primitives rather than introducing another parallel styling layer

**User-testable exit condition:** a tester can move through the current shared application surfaces and see consistent headers, actions, cards, panels, and common control styling, and engineers can extend the same component layer for the next workflow-runtime sprint without inventing a new surface pattern each time.

### UI Overhaul 2.0: Out-Of-Roadmap UX Detour (Complete)

**Outcome:** the application shell and already-delivered route surfaces were realigned with the approved UI guidance before new roadmap feature scope resumed.

**Build:**

- rebuilt the shared authenticated shell around the approved product-first navigation posture
- moved account, scope, delegation, sign-out, and theme affordances into the sidebar footer context area
- added shell-level access-denied feedback and redirected unauthorized deep links back to Home
- kept sign-in outside the authenticated application shell
- shifted Home toward queue-first operational work instead of destination-launcher cards
- moved Organization toward a quieter scope-aware explorer posture
- added section description and column-count support for form sections
- rebuilt the form builder around stacked section panels and section-level controls
- aligned Workflows and Responses to the shared shell posture without adding new roadmap product scope
- refreshed closeout expectations so smoke and UAT validate the new shell contract

**Application UI delivered this detour:**

- one coherent authenticated shell for product and internal routes
- native route ownership for the major product surfaces already delivered before the detour
- Components and Datasets exposed as native inspection surfaces
- Reports retained only as a transitional compatibility surface

**User-testable exit condition:** a tester can sign in, move through Home, Organization, Forms, Workflows, Responses, Components, Dashboards, Administration, and Migration under the updated shell, exercise form-section authoring, and see unauthorized deep links return to Home with transient feedback.

## Phase 2: Workflow Runtime, Responses, And Materialization

### Sprint 2A: Workflow Assignment And Response Start (Complete)

**Outcome:** published forms and workflows are assignable and discoverable from the product UI.

**Build:**

- workflow-assignment flows
- response-start entry points
- scope-aware pending-work surfaces
- first-step-only workflow runtime foundation that can be extended without replacing the Sprint 2A data model

**Application UI delivered this sprint:**

- usable assignment flows
- clear "start response" entry points in the intended application shell
- migration of the Sprint 2A-touched `Home`, `Forms`, `Workflows`, and `Responses` surfaces off the hybrid shell and onto native SSR ownership with successful hydration

**User-testable exit condition:** a tester can assign work and start the correct response flow without builder tooling, while the runtime foundation remains ready for later multi-step expansion.

### Sprint 2B: Authentication Hardening And Settled-Surface Native SSR Slice (Complete)

**Outcome:** authentication and session behavior are safe enough for broader internal testing, and the most settled product routes no longer depend on the hybrid shell.

**Build:**

- replace plaintext password comparison with Argon2id password-hash verification
- add password-hash migration and backfill for seeded and demo accounts plus user create and edit flows
- add session expiry, revocation, last-seen tracking, and logout invalidation semantics
- keep browser UI authentication on a same-origin `HttpOnly` cookie session contract while keeping bearer tokens only for explicit scripted access
- introduce a central authenticated-account extractor and request-context boundary instead of ad hoc header parsing in handlers
- replace raw internal and database error exposure with stable auth and session error responses plus traceable server logs
- keep these settled routes on native SSR ownership: `/login`, `/`, `/organization*`, `/forms*`
- remove shipped demo passwords from the public login surface while keeping local-development guidance in docs or internal-only tooling
- stop adding inline action handlers for newly migrated shared UI surfaces

**Application UI delivered this sprint:**

- native SSR login, home, organization, and forms surfaces with successful hydration and no bridge dependency
- stable sign-in, sign-out, and reload behavior through the intended application shell

**User-testable exit condition:** a tester can sign in, refresh, browse Organization and Forms, create or edit a form, publish a version, and sign out through native SSR-owned routes without touching the retained hybrid shell.

### Sprint 2C: Workflow/Response Backend Decomposition And Runtime Hardening Slice (Complete)

**Outcome:** workflow and response-entry behavior is reorganized enough that later workflow and response work no longer compounds the god-file pattern, while the native route ownership pulled forward by UI Overhaul 2.0 remains stable.

**Build:**

- preserve native route ownership for `/workflows*`, `/responses*`, response-start and resume entry surfaces, and administration links
- do not restore `/app/admin`; administration work belongs under `/administration*`
- decompose touched backend slices into bounded-context modules, starting with `workflows` and `submissions` and continuing the `auth`, `hierarchy`, and `forms` movement already started
- keep `tessara-api::lib` as router, middleware, and state composition only; no new workflow or business orchestration should land there
- move transport decoding and response shaping into handlers, orchestration into services, and SQL into repositories for touched slices
- add targeted integration suites for auth and session behavior, role and capability boundaries, form publish safeguards, workflow assignment, and response-start flows
- tighten shared UI primitives used by migrated routes so new SSR surfaces stop depending on raw inline `onclick` strings
- close the remaining workflow-assignment authorization gap so operators can only start assignments inside effective scope
- add a negative regression proving a scoped operator cannot start another account's out-of-scope workflow assignment by UUID

**Application UI delivered this sprint:**

- workflow browse, detail, assignment, response-start, and resume entry flows remain under native SSR ownership while backend seams move underneath them
- visible error and permission behavior remains stable under the UI Overhaul 2.0 shell

**User-testable exit condition:** a tester can browse workflows, assign work, start or resume the correct response entry flow, and verify role/scope boundaries without falling back to the retained hybrid shell.

### Sprint 2D: Draft, Submit, And Review Response Slice (Complete)

**Outcome:** the end-user response lifecycle is coherent and complete.

**Build:**

- pending, draft, submitted, and read-only review flows
- strict submit behavior
- canonical response persistence surfaced through application flows
- response edit, save, submit, and review routes delivered as native SSR from first delivery with no new bridge fallback
- touched `submissions` and workflow-runtime code continuing the `handler`, `service`, and `repo` split introduced in Sprint 2C
- browser response lifecycle flows supported only through the settled auth and session contract delivered in Sprint 2B
- finish moving response-facing auth/session use onto `AuthenticatedRequest` or config-aware helpers so customized browser cookie names work across touched flows
- keep bearer-token responses reserved for explicit script/test/API flows rather than normal browser sign-in behavior

**Application UI delivered this sprint:**

- polished Responses area aligned to the intended end-user-facing experience

**User-testable exit condition:** a tester can save draft, resume, submit, and review responses through the application UI.

### Sprint 2E: Multi-Step Workflow Authoring And Execution (Complete)

**Outcome:** workflows are no longer limited to a single response step, and runtime execution can advance across explicit step definitions.

**Build:**

- multi-step workflow version authoring with ordered step definitions
- explicit step transitions and runtime progression across workflow instances
- assignment support for step-specific work rather than only workflow-level single-step work
- contextual assignment creation from organization nodes and the global assignment console, backed by shared candidate/eligibility APIs
- response handoff behavior between steps, including completion of one step and activation of the next
- publish-time validation that multi-step workflow versions are structurally complete
- multi-step runtime work extending the decomposed workflow and runtime service layer rather than adding new orchestration to giant route modules
- typed workflow step and runtime states where touched, avoiding fresh stringly-typed state expansion
- touched workflow screens remaining native SSR and not reintroducing bridge-owned state management

**Application UI delivered this sprint:**

- workflow authoring screens that let operators define and inspect multi-step workflow versions
- assignment creation surfaces that let operators select valid `Node path - Workflow` candidates, use `Assign Workflow` from a selected organization node, and choose only valid assignees
- runtime surfaces that show current step, upcoming step, and completed-step history for in-flight work

**User-testable exit condition:** a tester can create a workflow with more than one step, assign it from both an organization node and the global assignment console using only valid node/workflow/assignee combinations, start it, complete the first step, and observe the next step become the active work item through the application UI.

### Post-Sprint 2E Design Detour: Rust/UI Styling And Component Alignment (Complete)

**Outcome:** the remaining workflow, assignment, response, home, and administration UX feedback gathered during Sprint 2E was consolidated into a coherent component, table, permissions, and stylesheet direction before the next functionality sprint.

**Already landed after the refresh:**

- form-first assignment now routes through generated single-form workflows and normal workflow assignment mechanics
- workflow assignments are the single source of truth for response starts and submission access
- generated workflow availability and assignment summaries have been refined for operator-facing selection
- stale workflow/form assignment DTO fields and direct submission assignment fields have been removed
- native shared UI primitives now cover buttons, icon buttons, status badges, data tables, searchable tables, filters, and common form action containers
- Playwright and standard validation wrappers are available through `scripts/validate-e2e.ps1` and `scripts/validate.ps1`

**Completed reconciliation:**

- table, queue, picker, and detail-readout surfaces now follow the canonical shared UI guidance and the current native SSR shell posture
- workflow, assignment, response, form, organization, administration user, and administration role tables have approved pagination, row-count, search/filter, mobile, and action treatments where applicable
- RBAC route coverage cleanup landed after the detour planning notes and updated the permissions scenario documentation
- `style/main.css` remains the documented active stylesheet entrypoint through the Cargo Leptos pipeline for the next functionality sprint
- deployed selector verification remains part of the Sprint 2F validation story rather than a standalone blocker
- residual UX polish is carried forward only when Sprint 2F touches the same runtime, materialization, or monitoring surfaces

**Application UI delivered this detour:**

- workflow directory, assignment directory, response, and home tables aligned to the selected Rust/UI table language where the current UI still uses page-local table controls
- workflow step editing controls, assignment assignee chips, status badges, and icon buttons using a consistent component vocabulary
- response and home work queues visually prepared for later delegated-work redesign without changing assignment/runtime rules
- stylesheet delivery and deployed-selector verification documented enough that UI edits can be validated without manual asset-path guessing

**User-testable exit condition:** a tester can browse the touched workflow, assignment, response, home, forms, organization, and administration routes and see consistent Rust/UI-style tables, tags, icon actions, access behavior, and form button spacing while all Sprint 2E workflow behavior and the generated single-form workflow shortcut remain intact.

### Sprint 2F: Runtime Status And Materialization Slice (Complete)

**Outcome:** runtime execution and materialization readiness are visible and usable.

**Build:**

- workflow/runtime status visibility
- materialization readiness and refresh status
- operator-facing monitoring screens
- CI enforcement for documented checks including `fmt`, `check`, wasm hydrate check, `test`, `clippy`, smoke, and legacy import rehearsal
- CI enforcement for `cargo audit`, with RustSec advisories upgraded away where possible and any accepted advisory exceptions documented with reachability analysis
- maintenance, import, and demo commands split away from HTTP startup so server startup and operational tooling are no longer conflated
- workflow-aware tracing and stable operator-facing error and reporting behavior
- hydration and browser-console cleanliness verified during UAT closeout for touched runtime and materialization routes

**Application UI delivered this sprint:**

- coherent internal runtime and materialization surfaces that do not disrupt the main user shell

**User-testable exit condition:** operators can inspect runtime and readiness through the app while end-user flows remain working.

## Phase 3: Dataset Engine And Revisions

### Sprint 3A: Dataset Authoring Foundation Slice (Complete)

**Outcome:** datasets become first-class application assets for practical v1 authoring and preview.

**Build:**

- dataset directory/detail/create/edit flows
- source composition using published form sources and reusable dataset expression controls
- field projection, grouping/aggregation controls, generated SQL preview, and filters placeholder in the final authoring flow
- clearer separation between authoring and viewing surfaces
- stable logical form field identity with dataset SQL generated against `(form_version_id, field_id)` instead of mutable field keys
- dataset and reporting work following bounded-context backend structure on touch
- query planning and execution concerns moving behind clearer dataset and reporting service boundaries
- pagination, limits, and guardrails added to dataset and reporting list and execution surfaces where touched
- dataset visibility guarantees for every dataset preview surface touched here, including negative coverage that no-access users cannot read dataset APIs
- row filters and calculated fields are intentionally deferred to the follow-on advanced authoring slice
- explicit dataset restriction filters/rules, including future row-level node restrictions and custom capability hooks, are intentionally deferred

**Application UI delivered this sprint:**

- usable dataset authoring screens in the application

**User-testable exit condition:** a tester can create, inspect, edit, and preview datasets through app UI, while scoped operators can read the full materialized output for datasets visible to their effective scope.

### Sprint 3B: Dataset Advanced Authoring Slice (Complete)

**Outcome:** dataset authors can refine datasets beyond direct source-field projection.

**Build:**

- row filter authoring for dataset sources or dataset output, with clear UI validation and preview behavior
- explicit dataset restriction filters/rules, including possible row-level node restrictions and custom capabilities, so richer access behavior is deliberately authored instead of implied by system metadata
- calculated field authoring for v1-safe expressions over selected source fields
- typed validation and error states for invalid filters, missing field references, and unsupported calculated-field expressions
- preview execution that applies filters and calculated fields consistently with saved definitions

**Application UI delivered this sprint:**

- dataset edit screens expose row filters and calculated fields without changing the basic Sprint 3A authoring workflow

**User-testable exit condition:** a tester can add a row filter and calculated field to a dataset, preview the resulting rows, save the definition, and verify any explicit restriction rules behave as authored.

### Sprint 3C: Dataset Revision And Compatibility Slice (Complete)

**Outcome:** revision behavior is visible and manageable.

**Build:**

- revision publishing and revision history
- compatibility findings
- carry-forward behavior
- dependency visibility
- revision, compatibility, and dependency states normalized into typed values rather than expanded raw string comparisons
- dependency and compatibility results surfaced through typed contracts that later component and dashboard work can consume directly

**Application UI delivered this sprint:**

- revision history, detail, and compatibility screens

**User-testable exit condition:** a tester can revise a dataset and understand downstream impact from the UI.

## Phase 4: Components

### Sprint 4A: Dataset Catalog And Thin Table Components (Complete)

**Outcome:** Dataset discovery improves and table-oriented presentation assets become thin display components over Dataset major-line outputs.

**Build:**

- implement Component frontend surfaces in a dedicated `tessara-web-components` crate from the start, with root `tessara-web` retaining route adapters, shell/auth/session/navigation policy, hydration, document integration, CSS, and assets
- one public `table` component kind with one last-mile projection, one saved default filter set, default sort, page size, optional search fields, and display-label overrides
- Dataset catalog tags, provenance lineage, and searchable Dataset directory/detail surfaces so authors can choose display-ready Datasets as the source of truth
- component versioning and publication
- edit-screen version decisions: update existing published version in place or create a new version through a consumer-review modal with a version note
- validation and Dataset major-line binding behavior; component versions do not bind Dataset revisions directly
- table-only component-version storage enforced by schema constraint and squashed into the baseline migration for the sprint reset model
- any retained legacy analytical endpoints stay adapter-only; no new core behavior may deepen deprecated asset families
- touched reporting and component routes continuing hybrid-shell removal rather than creating a second long-lived bridge
- component list/detail endpoints enforce scoped dataset and component visibility with negative operator coverage

**Application UI delivered this sprint:**

- component directory/detail/create/edit/versions/view flows
- edit-screen publishing with Update Existing Version and Create New Version actions
- table viewers inside the application using the shared interactive table display
- Dataset catalog search, tag editing, provenance lineage, and Dataset context in Component authoring surfaces

**User-testable exit condition:** a tester can tag and discover Datasets, review direct provenance, then create, version, publish, and view thin table components in the app.

### Sprint 4B: Chart And Stat Component Slice (Complete)

**Outcome:** visual presentation assets are first-class components.

**Build:**

- `Bar`, `Line`, `Pie/Donut`, and `StatCard` authoring
- component-specific validation and viewing behavior
- visual component authoring and viewing built directly on `ComponentVersion` and typed validation state
- any retained legacy visual-analysis endpoint kept explicitly adapter-only
- any legacy adapter endpoint touched here must enforce scoped component and dashboard visibility before returning metadata

**Application UI delivered this sprint:**

- visual component builder and viewer screens

**User-testable exit condition:** a tester can build and view visual components without deprecated workbench flows.

## Phase 5: Dashboard Composition

### Sprint 5A: Dashboard Composition Slice (Complete)

**Outcome:** dashboards compose component versions through application-grade flows.

**Build:**

- dashboard directory/detail/create/edit/view flows
- component placement and composition
- clearer product-facing dashboard viewers
- dashboard composition depending on `ComponentVersion`, not legacy report or chart nouns
- touched dashboard routes remaining native SSR and not reviving product-facing bridge logic
- dashboard viewer and composition endpoints preserve scoped component visibility for operators

**Application UI delivered this sprint:**

- readable product-facing dashboard screens
- usable internal dashboard composition screens

**User-testable exit condition:** a tester can assemble and view dashboards through the app.

## Phase 6: Modular Application Platform Foundation

### Sprint 6A: Module Contract And Core Control Plane Slice (Complete)

**Outcome:** the current Tessara application is represented and administered as Core plus discoverable transition contributions, with future real Module Release/Instance wire types defined but no real-instance persistence or mutation, and without pretending in-process features are deployable Module Releases.

**Build:**

- define `ApplicationInstallation`, `CoreRelease` (including its gateway component), `ModuleDefinition`, `ModuleRelease`, `ModuleInstance`, Feature Declaration, functional contract, typed resource reference, semantic destination, and module contribution concepts
- introduce a versioned manifest covering identity, version, Core Release plus Shell Context/UI SDK/design-system compatibility, `tessara-oci-v1` runtime/migration image digests and execution/configuration/probe/shutdown/resource declarations, machine-readable Feature Declarations, functional dependencies/provider-binding constraints, required and provided contracts, resource types, product and administration routes, navigation plus optional Home/work-discovery/search contributions, namespaced security capabilities, configuration schema, and health/readiness contracts
- model current in-process feature areas with explicitly non-installable `transitional_in_process` contribution descriptors that may reserve future Module Definition identities but create no Module Release/Instance, cannot satisfy dependencies as module providers, and cannot be materialized by the Supervisor; Sprint 6B's reference module is the first full `tessara-oci-v1` manifest
- keep functional contracts distinct from security capabilities, and keep Core authoritative for roles, assignments, organization scope, and access/composition audit while modules retain product audit
- keep Definition `registered`; Release `trusted`/`compatible`; Instance `live`/`tombstoned`, `installed`, `deployed`, `configured`, `ready`, `enabled`, `healthy`, and `retained`/`destroyed`; Navigation Contribution display policy; and actor/action/resource/scope authorization as explicit separate dimensions
- add Core persistence and services for module inventory, manifest validation, dependency findings, Feature Declaration and security-capability discovery, and navigation policy
- introduce installation-scoped typed resource references whose owner, type, and identifier cannot be reinterpreted
- define typed resolution and state-observation outcomes without imposing module product rules such as immutability or when a new version must be published
- introduce semantic named-route destinations instead of hard-coded cross-module URLs
- expose current Forms, Workflows, Responses, Datasets, Components, and Dashboards through temporary in-process contribution descriptors and typed adapters; expose Migration only as a retired historical/support descriptor
- update permission scenarios for module administration, contributed security capabilities, and navigation configuration
- define real Module Release/Instance public types only; their persistence, mutation, and materialization begin in Sprint 6B
- classify transitional Migration as retired, with no current route, navigation contribution, provider claim, or fabricated destination; restoration requires a new product decision and roadmap scope
- keep navigation reordering inside immutable Core-assigned bands—Forms/Workflows/Responses between Organization and Operations, Components/Dashboards after Operations, and Datasets between Administration and the new fixed Module Management anchor—with permanent Core destinations policy-immutable; cross-anchor movement and grouping changes remain deferred
- make `modules:read` and `modules:manage_navigation` installation-global, with manage implying read and `admin:all` implying both
- reject roles that mix scope-aware and installation-global capabilities except for the sole universal-sentinel `admin:all` case; classify that complete exception role as installation-global, keep every ordinary mixed bundle invalid, and retain the cleaner `admin:all`-only built-in admin seed contract
- add policy-immutable Core Module Management to the `Admin` group after Datasets, visible with effective global `modules:read`; retain the separate `admin:all`-only Administration item and gate navigation mutation controls on `modules:manage_navigation`

**Application UI delivered this sprint:**

- a Core-owned module directory and detail/status experience reached through a fixed `Admin` navigation item for effective global `modules:read`
- Feature Declarations and their use cases, inputs, outcomes, constraints, and realizing contracts visible for human and machine discovery
- navigation-policy readback for global `modules:read`, with visibility/order mutation controls enabled only for global `modules:manage_navigation`
- module-provided security capabilities visible in Core role management
- dependency, compatibility, configuration, readiness, and health findings presented separately

**User-testable exit condition:** an actor with global `modules:read` can discover fixed Module Management in the `Admin` group and inspect Forms, Workflows, Responses, Datasets, Components, and Dashboards as clearly labeled in-process contributions rather than Module Instances; understand their advertised Feature Declarations; and review their contracts, security capabilities, and current navigation policy without receiving mutation controls. An actor with global `modules:manage_navigation` can change contribution visibility/order without changing authorization. The same directory shows Migration as retired historical/support inventory with no route, navigation item, provider, Feature Declaration, contract, capability, or action, while every current product route continues to work.

### Sprint 6A-UI: Navigation Composition And Module Management Harmonization Slice (Next)

**Outcome:** Tessara navigation is composed from revisioned, configuration-driven ordered groups and item placements instead of hard-coded `Main`/`Admin` sections, Core anchors, and reorder bands. Core protects its required groups and destinations while administrators can manage custom groups and freely place optional destinations. The redundant `/administration` page and route are removed, and the Sprint 6A Module Management surfaces are harmonized with the existing Tessara UI.

**Relationship to Sprint 6A:** this is a new post-closeout sprint based on the closed Sprint 6A commit. It does not reopen, amend, or replace Sprint 6A's implementation, closeout report, rollback package, or acceptance evidence. Sprint 6B follows this slice with its runtime scope unchanged.

**Build:**

- introduce a forward-only migration from the Sprint 6A contribution-only band policy to a revisioned group-and-placement model covering Core and contributed destinations; preserve every compatible visibility/order choice and reject ambiguous or orphaned state
- define a Core-owned built-in destination catalog with stable identities, routes, capability requirements, default placements, and independent protection flags for removal, visibility, and group movement
- require stable `core.main` and `core.admin` groups; protect Home in Main, allow Organization to be hidden but not removed or moved out of Main, and protect User Management, Roles & Access, Node Types, and Module Management in Admin while allowing all protected items to reorder inside their group
- let effective global `modules:manage_navigation` create, rename, delete, and reorder custom groups and show, hide, reorder, or move every non-protected destination between groups; block deletion of non-empty groups and preserve atomic revision-conflict/audit behavior
- support configurable labels for custom groups; required-group display-label editing is permitted but is not an exit-critical requirement, while stable Core group identities never change
- remove the Administration navigation item, page component, and `/administration` route without a compatibility redirect or tombstone; the exact path receives the ordinary unmatched 404, while `/administration/users`, `/administration/roles`, `/administration/node-types`, and `/administration/modules` remain direct Core Admin destinations with their existing authorization
- replace navigation-policy schema v1 and shell-navigation schema v1 assumptions with versioned group-aware wires and fail-closed validation that accepts arbitrary valid groups/order without trusting display configuration as authorization
- keep capability eligibility, module availability, direct-route/API guards, `modules:manage_navigation` implying `modules:read`, `admin:all` implying both global module capabilities and remaining the sole universal-sentinel mixed-scope exception, and display-policy/authorization separation unchanged
- audit and harmonize the Module Management directory/detail, group-management controls, and capability-provenance presentation using the existing Tessara identity and native Leptos patterns; do not redesign unrelated product workflows
- preserve native Leptos SSR, hydration, direct-load/refresh ownership, useful no-JavaScript documents, clean browser consoles, semantic HTML, and the prohibition on `/bridge/*`, route-level HTML-string injection, and JavaScript controller ownership
- preserve tests as durable proof: record every intentionally superseded Sprint 6A assertion before editing it, retain all unrelated identities unchanged, and add equal-or-stronger migration, invariant, authorization, group CRUD, cross-group placement, concurrency, SSR, accessibility, and viewport proof
- use proportional targeted validation during implementation, then run the reconciled complete browser inventory, fresh and populated-upgrade deployment proof, smoke/UAT, rollback compatibility, SSR/hydration/console, accessibility, responsive, and source-quality gates against the closing commit
- keep Module Release/Instance persistence, materialization, Supervisor, gateway, OCI, module databases, and runtime work out of scope; Sprint 6B remains unchanged

**Application UI delivered this sprint:**

- direct Core Admin destinations for User Management, Roles & Access, Node Types, and Module Management, with no redundant Administration landing page or route
- an accessible group-and-item navigation composer for readers and managers, including custom-group CRUD, group order, optional-item visibility, cross-group placement, and item order
- Module Management directory/detail and capability-provenance presentation aligned with existing Tessara patterns
- responsive, keyboard, focus, SSR/no-JavaScript, and narrowly reviewed visual proof for the changed navigation and Module Management surfaces

**User-testable exit condition:** a global Module Management reader can inspect the navigation configuration without mutation controls. A global navigation manager can create, rename, reorder, and delete empty custom groups; reorder Main and Admin; move any optional destination between groups; and show, hide, or reorder optional destinations without changing authorization. Main, Admin, Home, Organization, User Management, Roles & Access, Node Types, and Module Management enforce their approved protection rules. `/administration` has no route or navigation item. The migrated policy preserves compatible prior choices, invalid mutations fail atomically with stable audited errors, the harmonized Module Management pages remain complete and responsive, and the reconciled acceptance inventory passes without skips, retries, weakened unrelated expectations, or unexplained test rewrites.

### Sprint 6B: Module Runtime And Installation Infrastructure Slice

**Outcome:** Tessara can securely materialize a Core Release and install/operate an independently deployed full-stack module inside one Supervisor-rooted application installation.

**Build:**

- introduce same-origin gateway routing and service discovery for separately deployed module UI and API routes
- define the foundational deterministic Materialization Plan schema/digest plus separate Apply Authorization Envelope in 6B, with a hand-authored resolved lockfile/plan fixture, detached signature, first-apply envelope, and verifier; the Supervisor consumes this stable contract now and Sprint 6D embeds the same plan unchanged in generated Application Lockfiles
- introduce an installation-local Supervisor process and bootstrap CLI outside Core; before Core exists it creates the stable installation identifier, trust anchors, and authoritative ledger, then owns locked Core Release component (including gateway) and Module Release materialization, database provisioning, migrations, health gates, traffic switching, rollback, and recovery
- define a versioned Core Administration Capability Floor and require every resolved composition to designate an Administrator Enrollment Role that includes the floor; reject missing or weakened designations, assign it globally during enrollment, and use the same active/authenticable identity plus global floor predicate to determine whether a Viable Core Administrator exists
- after first Core health, let the Supervisor issue installation-bound, single-use, expiring Administrator Enrollment Claims under local operator/host authorization and a current Core decision that no Viable Core Administrator exists: `initial` only until one has ever existed, and `recovery` only with additional explicit audited break-glass authorization
- allow at most one issued or reserved claim generation per installation; implement `issued -> reserved -> consumed` plus expiry/revocation, make replacement revoke the prior generation, persist only a one-way verifier and non-secret lifecycle/reservation metadata, and show the secret once outside Blueprints, lockfiles, receipts, logs, diagnostics, status/read-back, Core audit, and recovery output
- implement idempotent enrollment: Core reserves the current generation with the Supervisor, locally transacts identity create/bind plus global designated-role assignment plus redemption record, and returns a signed result for Supervisor consumption/reconciliation; every redemption checks current Supervisor state so restore of a pre-redemption Core backup cannot revive an old claim
- require mutually authenticated Core/CLI-to-Supervisor handoff bound to installation, base receipt, target plan digest, monotonic desired/apply revision, nonce/idempotency key, initiator/approver evidence, expiry, and destructive-action scope; reject replay/stale/concurrent plans, serialize mutation, and reconcile the authoritative ledger into Core after startup
- define a versioned authenticated Shell Context and SDK through which each route-owning module server-renders a complete coherent document, plus a Core-rendered gateway fallback when that module is unavailable
- propagate verifiable installation and user context plus scope-bound Authorization Grants or exact Core decisions while preserving server-managed browser sessions; prohibit independent capability/scope sets
- define descendant-aware scope expansion, Organization/authorization revision freshness, delegation/ownership assertions, and per-downstream-audience exchange bound to the original actor, presenting service identity, declared dependency/contract/action, and installation without sharing browser cookies, bearer credentials, or database credentials between modules; restrict service-only grants to explicitly authorized system jobs
- enforce the v1 single-cluster installation topology and provision one PostgreSQL database per module instance beside the Core database; do not add multi-cluster or module-selected external relational database placement
- provision separate runtime and migration roles, scoped credentials, module-owned migrations, and explicit undeploy/reactivation/data-destruction behavior that preserves or tombstones durable Module Instance identity correctly
- implement the sole v1 `tessara-oci-v1` Deployment Profile and Supervisor adapter: digest-pinned runtime/optional migration images, platform/architecture, commands, listen/service registration, config/secret injection, separate runtime/migration identities, probes, graceful shutdown, and resource declarations; reject unsupported profiles and conformance-test every field
- add trusted artifact-source configuration, publisher/signature or provenance verification, and digest-pinned OCI acquisition without relying on Core to replace itself
- define the gateway as a separately running component artifact versioned and selected only inside its Core Release; prove a Supervisor-driven Core Release patch upgrade/rollback, including its gateway, while Supervisor status remains available, and define the Supervisor's separate non-self-replacing upgrade procedure
- make upgrade select a new compatible Module Release while preserving the Module Instance identifier, database binding, references, configuration identity, and rollback record
- prohibit cross-module database access, cross-database foreign keys, shared writable schemas, FDW shortcuts, and shared runtime credentials
- define module SDK conventions for manifests, `tessara-oci-v1`, configuration APIs, administration UI, route integration, request context, health/readiness, diagnostics, generated clients, and conformance tests
- require human and machine configuration to use the same module-owned validation contract
- add local multi-process development, testing, startup, shutdown, and diagnostic workflows
- implement explicit timeout, unavailable, disabled, and degraded-state behavior at the gateway and shell
- keep Core Module Management and eligible module administration/configuration/diagnostics destinations reachable while a product destination is disabled, unconfigured, or unhealthy
- ship a full-stack conformance/reference module with its own administration screen and database before moving a product feature
- define the conformance timing methodology, normalized test environment, sample size, and pass/fail tolerance for known-versus-random non-disclosure tests
- add negative conformance scenarios proving (1) one actor with capability A only for subtree X and capability B only for subtree Y cannot obtain A/Y or B/X, (2) undeclared or declared-but-wrong-audience/action services cannot exchange or replay that actor's authority, (3) grants/exchanges issued before role, subtree, ownership, or delegation revision changes are rejected as stale without leaking existence, and (4) unauthorized resolution of known and random identifiers is indistinguishable under the defined status/shape/timing contract

**Application UI delivered this sprint:**

- installation, configuration, enablement, health, and diagnostics flows for the reference module
- read-only Supervisor ledger, active Materialization Plan/apply, and Core Release status in Core administration
- a one-time administrator-enrollment surface distinct from normal sign-in, with local-user and external-identity paths, initial/recovery context, capability-floor-safe assignment, and no secret redisplay
- coherent unavailable and disabled states in navigation and routing

**User-testable exit condition:** using the signed 6B release fixture, a tester can establish the Supervisor-rooted installation; use a once-displayed initial claim to enroll a Viable Core Administrator into the locked floor-compliant role; prove concurrent/replacement, replay, expiry, revocation, cross-installation, interrupted reconciliation, and pre-redemption-restore cases fail or resume as specified; install/configure/authorize/enable/navigate to/disable/diagnose the `tessara-oci-v1` reference module; reject a replayed or stale apply envelope; and complete a Core Release patch upgrade/rollback. Module data remains isolated in its own database, and stopping it produces a contained degraded state rather than breaking Core.

### Sprint 6C: Independently Deployed Dashboard Module Slice

**Outcome:** Dashboards is the first existing Tessara feature area to operate as a separately deployed, full-stack module.

**Build:**

- move Dashboard UI, API, persistence, migrations, configuration, and health/readiness into the Dashboard module
- provision one Dashboard database per Dashboard module instance and remove Dashboard runtime access to Core or other module databases
- replace Dashboard database relationships to Components with typed `core_installation`-owned transition `ComponentVersion` references resolved through a versioned, first-party Core Release compatibility contract; do not masquerade the in-process contribution as a Module Instance
- expose required Component metadata and rendering/execution behavior through that typed adapter, mark the binding transition-only and unavailable to new external Blueprints, and require Sprint 8A to migrate data/references explicitly before retiring it
- obtain Core-issued, downstream-audience scope-bound grants or decisions bound to original actor, Dashboard service identity, the resolved Components dependency/contract/action, and installation rather than forwarding Dashboard authority
- advertise Dashboard routes, navigation, functional contracts, resources, configuration, diagnostics, and security capabilities through its manifest
- preserve the Sprint 5A authoring and viewing experience through same-origin routing
- show a non-disclosing forbidden placement for unauthorized resolution; after authorization, distinguish unavailable, inactive, superseded, provider-resource tombstoned, owner-module-instance tombstoned/data-destroyed, missing, and not-evaluated outcomes without redefining Component lifecycle semantics
- restructure seed and local data directly for the new database layout because no production compatibility obligation exists
- add contract, permission, outage, and browser coverage across Core, Dashboard, and the temporary in-process Components provider

**Application UI delivered this sprint:**

- the existing Dashboard directory, editor, detail, and viewer served by the independent module
- Dashboard configuration and diagnostics reached from Core module administration
- clear placement degradation when the Components provider is unavailable or changes lifecycle state

**User-testable exit condition:** a tester can compose and view a Dashboard through the normal shell while Dashboard runs in a separate process and database, and can observe contained behavior when Dashboard or Components is unavailable.

### Sprint 6D: Application Blueprint And Composition Automation Slice

**Outcome:** a Tessara application is a declarative, validated, reproducible composition suitable for human and LLM-driven construction.

**Build:**

- define a versioned Application Blueprint covering a Core Release version constraint, typed Core configuration such as Organization schema/terminology, selected modules, their version constraints and desired enablement, dependency bindings, module configuration, optional module-owned bootstrap declarations, navigation policy, Core-owned role definitions and role-to-capability mappings, a designated Administrator Enrollment Role that covers the Core Release's capability floor, and environment secret references
- add machine-readable module catalog discovery, Feature Declarations, contribution schemas, configuration schemas, and functional contract descriptions
- calculate dependency closure and reject missing, incompatible, cyclic, untrusted, or unbound compositions
- produce a lockfile containing the Blueprint revision/digest, exact Core Release version/component image digests including its gateway, exact Module Release image digests, selected Deployment Profile versions, resolved desired module enablement, composition-engine/schema and required Supervisor/deployment-adapter contract versions, contract/configuration/bootstrap schema versions, dependency bindings, resolved normalized non-secret Core/module configuration and navigation/role policy values plus their digests, designated Administrator Enrollment Role and Core Administration Capability Floor version, normalized bootstrap values or durable content-addressed bootstrap references plus digests, versioned secret-reference identities, and the deterministic 6B Materialization Plan plus digest and enable/disable actions
- implement deterministic validate, plan, diff, separate Apply Authorization Envelope creation, Supervisor handoff/apply, read-back, and conformance operations with idempotency and provenance; Core UI/API and all machine clients use the same handoff rather than an in-Core self-update path, and planning/LLM access does not imply approval authority
- ensure product artifacts are created or changed only through module-owned APIs
- let modules optionally expose typed, idempotent bootstrap/apply/read-back contracts for catalogs or initial product records; record their input digests and result receipts without creating a generic content-package model
- acquire referenced bootstrap inputs from a configured durable content-addressed source and verify their locked digest before module-owned apply/read-back
- expose the same composition operations through versioned APIs for UI, CLI, automation, and later MCP or agent adapters
- share one versioned Composition Engine between Core and the Supervisor bootstrap CLI; before Core exists, allow the CLI to resolve a Blueprint from trusted catalog inputs or verify a detached signature over a pre-resolved lockfile/plan digest, create the separate apply-authorization envelope, record local operator/host authorization and provenance, and seed Core's desired/resolved records after startup
- create a complete reference-application blueprint and at least one reduced composition that omits unneeded modules
- emit installation/release receipts containing the observed composition-engine and Supervisor/deployment-adapter versions plus desired/observed module enablement, and retain the resolved composition needed for support and reproduction
- make UI module enablement, configuration, navigation-policy, role-definition, enrollment-role designation, or declared bootstrap-state edits create a new Blueprint/input revision or explicit desired/actual drift with adopt/reconcile actions; allow immediate emergency disable only through a constrained non-destructive envelope as an audited reasoned/expiring Supervisor-ledger override that remains drift
- keep each installation locally operable without a required central Tessara SaaS control plane

**Application UI delivered this sprint:**

- an application-composition view showing desired/current modules and enablement, the capability-floor version and designated enrollment-role validation, emergency overrides, dependency findings, proposed changes, resolved versions, drift, and apply/adopt/reconcile results
- readable release inventory, Supervisor ledger/apply/restart status, pending approval and conflict/replay findings, and provenance for the active installation

**User-testable exit condition:** an administrator or machine client can bootstrap from a Blueprint or a detached signature over a resolved lockfile/plan digest; resolve two different Blueprints; separately approve them; hand their Materialization Plans and Apply Authorization Envelopes to the local Supervisor; reproduce their exact Core Release components/Module Releases/configuration/policy and declared bootstrap composition from lockfiles plus externally resolved secrets; survive a Core restart during apply; detect and reconcile a deliberate UI configuration change; and rerun an unchanged Blueprint as a no-op without handwritten deployment or database glue.

## Phase 7: Cross-Module Authorization And Resource Correctness

### Sprint 7A: Scoped Analytics And Cross-Module Authorization Slice

**Outcome:** dataset, component, and dashboard execution is scope-safe across the real Dashboard process boundary to the transition-only Core Components compatibility contract and the typed adapters later extractions retain.

**Build:**

- enforce explicit scoped restriction rules for dataset previews, component execution, and dashboard viewing when those rules are authored
- apply scoped metadata visibility to datasets, revisions, component versions, dashboard composition, and linked presentation assets
- propagate and verify installation, original actor, Dashboard presenting-service identity, declared Core compatibility binding/contract/action, scope-bound grants/Core decisions, freshness, and downstream audience across the real Dashboard process boundary and equivalent in-process Dataset/Component adapters
- add negative coverage proving scoped operators cannot see blocked rows, linked entities, metadata, or dashboard content, including mixed capabilities assigned to disjoint Organization subtrees; undeclared and wrong-audience/action services; stale grants after role/scope/ownership/delegation revision changes; and known-versus-random identifiers under the 6B non-disclosure profile
- keep retained deprecated analytical endpoints adapter-only and align their authorization with canonical module contracts when touched
- move touched Dataset and Component execution paths toward boundaries that can later move into their own deployments
- preserve clear empty, unavailable, and forbidden states without leaking metadata or internal authorization details

**Application UI delivered this sprint:**

- existing Dataset, Component, and Dashboard surfaces remain usable with corrected scoped behavior
- operators receive understandable scoped and cross-module failure states

**User-testable exit condition:** a scoped operator can preview Datasets, execute/view Components, and view Dashboards according to authored visibility and restriction rules across the real Dashboard boundary and transitional typed adapters, while an administrator sees the full seeded analytical set. Each Phase 8 extraction must rerun this proof when its adapter becomes a process boundary.

### Sprint 7B: Cross-Module Resource Lifecycle And Dependency Slice

**Outcome:** consumers can observe and respond to provider-owned resource changes through the real Dashboard-to-Core-compatibility boundary and reusable typed adapters, without owning provider product semantics or confusing transition ownership with the eventual module.

**Build:**

- complete typed resolution and revision/state-change contracts for Dataset, Component, and Dashboard relationships
- let each owning module define lifecycle states and decide which mutations require a new published version
- ensure consumers observe relevant changes through live resolution, revision markers, events, or an explicit combination declared by the provider
- implement changelog impacts, stale-dependency findings, carry-forward, and rebinding over durable typed references
- preserve reference type and owner when mutable characteristics such as active, inactive, or superseded change
- add publication or activation guards only where the owning module contract requires them
- distinguish tagged Core or module owner state, unknown/mismatched owners, owner-module-instance tombstoned/data-destroyed, provider-resource lifecycle (including archived or tombstoned), unavailable, authorization-not-evaluated, incompatible, unknown-resource, undisclosed, and not-evaluated outcomes internally; caller-visible projections must collapse all resource-specific detail to one restricted/undisclosed result whenever existence disclosure is not authorized
- add contract-version compatibility and consumer regression coverage

**Application UI delivered this sprint:**

- dependency health, observed provider state, upgrade, carry-forward, and rebinding flows
- clear choices to resolve or defer findings without Core inventing provider lifecycle policy

**User-testable exit condition:** a tester can change the lifecycle or revision state of a referenced Component resource, observe it across the Dashboard boundary, and resolve or defer resulting dependency findings. Equivalent Dataset/Component behavior is proven through the same contract adapters and must be rerun after physical extraction.

## Phase 8: Current Feature Module Separation

Every sprint in this phase must leave the extracted feature as a separately deployed full-stack module with its own administration/configuration surface, manifest, Feature Declarations, security capabilities, database, migrations, health/readiness, same-origin routes, and conformance coverage. Cross-module relationships use APIs, events, exports, and typed references only. Each extraction must inventory Core-owned transition references, migrate data into the new Module Instance, publish a complete old-to-new mapping, invoke versioned consumer-owned rebinding, emit completeness receipts, preserve explicit migrated/retired resolution for old references, and remove the read-only Core compatibility adapter only after all consumers are verified. Each extraction reruns the Phase 7 scope, lifecycle, outage, and compatibility proofs against the newly physical boundary.

### Sprint 8A: Component Module Separation Slice

**Outcome:** Components is independently deployed and consumes Datasets only through a public contract.

**Build:**

- move Component UI, API, versions, execution, persistence, configuration, and diagnostics into the Component module
- create the real Component Module Release/Instance; migrate Component data from Core; publish a complete old-Core-reference to new-Module-reference mapping; and invoke Dashboard-owned versioned rebinding so stored placements are rewritten with migration receipts before the Core adapter becomes read-only and is removed
- keep old `core_installation` transition references owner/type-stable with explicit migrated/retired resolution rather than silently interpreting them as Component Module references
- replace Component-to-Dataset database relationships with typed Core-compatibility Dataset references and versioned contracts until Dataset extraction performs the same explicit migration
- keep Dashboard-to-Component behavior on the public contract introduced in Sprint 6C
- migrate seed/test data and remove direct access to Dataset, Dashboard, or Core storage
- add dependency outage, scope propagation, capability, and compatibility coverage

**Application UI delivered this sprint:** unchanged Component authoring/viewing plus module-owned configuration and diagnostics through the shared shell.

**User-testable exit condition:** a tester can prove every Dashboard placement was explicitly rebound from its Core-owned transition reference to the new Component Module Instance with receipts, then author and execute Components against Dataset compatibility contracts across separate processes/databases while Dashboards continue to consume Components and degrade coherently during outages.

### Sprint 8B: Dataset Module Separation Slice

**Outcome:** Datasets is independently deployed and consumes source data through explicit provider contracts.

**Build:**

- move Dataset UI, API, revision, execution, materialization, persistence, configuration, and diagnostics into the Dataset module
- migrate Dataset data and Component-owned Dataset references from the Core compatibility owner to the real Dataset Module Instance through mapping/rebinding receipts before retiring the adapter
- replace reads of Response or other source tables with versioned source APIs, exports, and events
- preserve Dataset-to-Component contracts and scoped execution
- keep batch operations and catalog/template semantics owned by Datasets unless a future module provides a declared operation contract
- migrate seed/test data and remove direct access to Response, Component, or Core storage
- add materialization retry, dependency outage, scope, and compatibility coverage

**Application UI delivered this sprint:** unchanged Dataset directory, authoring, preview, status, configuration, and diagnostics surfaces through the shared shell.

**User-testable exit condition:** a tester can materialize and preview a Dataset from a provider contract, execute a Component over it, and view the result on a Dashboard across independently deployed modules.

### Sprint 8C: Response Module Separation Slice

**Outcome:** Responses is independently deployed and exposes captured data without sharing its persistence.

**Build:**

- move response start, draft, save, submit, review, export/materialization, persistence, configuration, and diagnostics into the Response module
- consume typed FormVersion and Workflow context without reading Forms or Workflow tables
- expose typed response/source contracts and events for Dataset and Workflow consumers
- preserve assignment-only response starts and scoped review behavior
- migrate seed/test data and remove direct access to Forms, Workflows, Datasets, or Core storage
- add cross-module authorization, idempotency, event, outage, and compatibility coverage

**Application UI delivered this sprint:** unchanged response-entry, submission, review, configuration, and diagnostics flows through the shared shell.

**User-testable exit condition:** a tester can complete and review a response through module contracts and consume its output in Datasets without shared database access.

### Sprint 8D: Workflow Module Separation Slice

**Outcome:** Workflows is independently deployed and coordinates Forms and Responses through public contracts.

**Build:**

- move Workflow authoring, versions, publication, assignments, runtime coordination, persistence, configuration, and diagnostics into the Workflow module
- replace Form database relationships with typed FormVersion references and provider contracts
- invoke Response runtime contracts and consume response events without sharing storage
- keep publication, assignment, handoff, and version policy owned by Workflows
- migrate seed/test data and remove direct access to Forms, Responses, or Core storage
- add authorization, retry/idempotency, outage, lifecycle, and compatibility coverage

**Application UI delivered this sprint:** unchanged Workflow authoring, assignment, execution, configuration, and diagnostics surfaces through the shared shell.

**User-testable exit condition:** a tester can publish and execute a Workflow over referenced FormVersions and separately deployed Responses while provider lifecycle changes and outages remain visible and contained.

### Sprint 8E: Forms Module Separation Slice

**Outcome:** Forms is independently deployed and is authoritative for Form and FormVersion product semantics.

**Build:**

- move Form and FormVersion authoring, publication, lifecycle, persistence, configuration, diagnostics, and catalog features into the Forms module
- expose typed Form and FormVersion resolution, state observation, and collection/rendering contracts
- keep decisions about mutations, publication, active/inactive/superseded behavior, and catalogs inside Forms
- ensure Workflow and Response consumers use provider contracts and never access Forms storage
- migrate seed/test data, remove Forms access to Core or consumer storage, and remove every other module's access to Forms storage
- add lifecycle, authorization, outage, compatibility, and catalog-contract coverage

**Application UI delivered this sprint:** unchanged Form directory, builder, version, publication, catalog, configuration, and diagnostics experiences through the shared shell.

**User-testable exit condition:** a tester can author and publish Forms, observe lifecycle changes through Workflow and Response consumers, and complete the reference application flow with every reference product module independently deployed. The former in-process Migration surface remains retired; Sprint 9A may introduce a separately approved module-owned migration coordinator rather than reactivating the retired transition implicitly.

## Phase 9: Migration, Hardening, And Modular Pilot Readiness

### Sprint 9A: Module-Owned Migration And Legacy Mapping Slice

**Outcome:** migration and import populate module-owned data through supported contracts and validate results through canonical module applications.

**Build:**

- align mapping documentation and verification with Core plus module-owned resources and databases
- route imports through each owning module's validation and import APIs rather than direct database writes
- use typed cross-module references and explicit binding steps during multi-module migration
- if a migration coordinator is separately approved, introduce it as a normal full-stack module with a reviewed identity, its own database, UI, manifest, Feature Declarations, security capabilities, and contracts; do not reactivate the retired generic Migration transition through a catalog edit
- update semantic route links, dry-run, idempotency, partial-failure, resume, and audit behavior for separately deployed modules
- inventory and remove remaining transitional reporting, hybrid-shell, legacy-builder, and shared-database paths
- reconcile canonical documentation references to absent archive sources

**Application UI delivered this sprint:** coherent module-owned import and verification experiences, plus a coordinator only if cross-module migration remains a supported capability.

**User-testable exit condition:** operators can dry-run, execute, resume, and verify approved module-owned import paths without an importer writing directly to another module's database. The retired in-process Migration contribution remains non-executable; any newly approved coordinator is independently deployed with its own reviewed identity, manifest, Feature Declarations, security capabilities, administration surface, and database.

### Sprint 9B: Modular Application Pilot Hardening Slice

**Outcome:** multiple independently deployed Tessara applications are reproducible, diagnosable, and stable for broader testing and separate support.

**Build:**

- run composition-level end-to-end, smoke, permission, scope, session, and performance coverage across at least two materially different blueprints
- rerun `tessara-oci-v1` acquisition, migration, runtime/configuration, service registration, probe, graceful-shutdown, identity, and resource-limit conformance for Core Release components and every selected Module Release
- test first bootstrap from both a Blueprint and a detached signature over a resolved lockfile/plan digest plus Supervisor-driven Core Release (including gateway component) and Module Release install, configuration, enable/disable, upgrade, compatibility rejection, failed migration, traffic switch, rollback, outage, recovery, undeploy/reactivation, explicit data destruction, and tombstone behavior
- test local-user and external-identity Administrator Enrollment Claim paths, capability-floor/designated-role validation, at-most-one generation and replacement revocation, reservation/local transaction/signed reconciliation and retry, once-only secret display, endpoint closure, expired/revoked/replayed/reserved/consumed/cross-installation rejection, audited recovery issuance, and restore of Core to a pre-redemption backup without claim reuse
- execute a separately managed Supervisor contract/binary and ledger-schema upgrade plus rollback through the bootstrap launcher, proving ledger backup/recovery and continued compatibility with the installed Core Release
- verify backup and restore for one module database and a complete application installation, including protected Supervisor Ledger/trust material, lockfiles/receipts, databases, and required external bootstrap inputs
- enforce connection budgets, query timeouts, health aggregation, and noisy-neighbor monitoring for the shared PostgreSQL cluster
- produce exportable diagnostic bundles and exact composition-engine, Supervisor/deployment-adapter, Core Release/component (including gateway), and Module Release version/artifact inventories
- document supported combinations, unsupported-v1 behavior, and application-specific release procedures
- replace permissive production CORS with same-origin or environment-specific policy suitable for cookie sessions
- verify browser authentication does not expose bearer tokens except through explicit script, test, or API-token flows
- verify no primary route depends on HTML-string shells, `/bridge/*`, shared product databases, or legacy adapter assets

**Application UI delivered this sprint:** coherent degraded, maintenance, compatibility, and recovery states across supported module compositions; no new primary surface is required.

**User-testable exit condition:** two different locked application compositions can be bootstrapped, installed, exercised, upgraded across Core Release components and Module Releases, interrupted, rolled back, recovered, backed up, restored, and diagnosed independently through their intended UI and local Supervisor/operational flows.

## Deferred Beyond This Roadmap

- printable report artifacts composed from prose and components
- full visual dashboard designer beyond the required composition flows
- fuzzy joins, complex window functions, and other analytical features not required for v1
- broader home-surface specialization after the shared shell and role-ready flows are stable
- Leptos lazy-loading/code-splitting pilot for one extracted route area before broader bundle splitting
- a required centralized multi-tenant Tessara SaaS control plane
- hot-swappable equivalent module providers and automatic transfer of provider-owned data
- a generic content-package abstraction separate from module-owned catalogs, templates, instruments, and batch definitions
