# Tessara Roadmap

This roadmap is authoritative as of June 5, 2026. It starts from the current implemented baseline after the native UI refresh and RBAC route coverage cleanup, identifies the transition from the current reporting stack to the re-aligned target model, and defines future delivery as explicit vertical-slice sprints.

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

- Run a local deployment refresh with:
  - `.\scripts\local-launch.ps1` for standard updates, or `.\scripts\local-launch.ps1 -FreshData` when the local UAT dataset should be reseeded from scratch.
- Print and run the sprint UAT script:
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Run Playwright browser validation, including the permissions scenario suite:
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- Confirm the UAT script output includes current route ownership and role-gated behavior before closing the sprint.
- Confirm any sprint that adds or changes permission-controlled behavior updates `docs/playwright-permissions-scenarios.md` and includes positive and negative Playwright coverage where currently executable.
- Confirm every route surface touched in the sprint remains under native SSR ownership before closing the sprint.
- Confirm route ownership, hydration, and browser-console cleanliness for every touched route before closing the sprint.
- If a detour sprint lands outside the numbered roadmap, reconcile this file with the codebase before selecting the next roadmap sprint.

## Cross-Cutting Delivery Constraints

- No new user-facing behavior may be added through HTML-string route shells, `inner_html` injection, `/bridge/*`, or retained legacy bridge assets.
- Any sprint that touches `auth`, `hierarchy`, `forms`, `workflows`, `submissions`, or `reporting` must move touched backend behavior toward bounded-context structure with explicit `router`, `handlers`, `service`, `repo`, and `dto` boundaries rather than expanding large vertical files.
- Browser authentication for native UI routes must use a server-managed session contract. JavaScript-managed bearer tokens may remain only for explicit CLI, script, or testing flows.
- Client-visible error payloads must use stable application codes and messages. Raw database and internal error strings must not be exposed to end users.
- Any sprint that exposes scoped analytical, workflow, response, dataset, component, chart, or dashboard data must prove operator scope filtering with negative regression coverage before closeout.
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
- a Leptos SSR shell with root-level native product routes for Home, Organization, Forms, Workflows, Responses, Components, Datasets, Dashboards, Administration, and Migration
- Sprint 2B authentication hardening: Argon2id credential storage, server-side session expiry/revocation/last-seen tracking, same-origin `HttpOnly` browser cookies, stable auth/session errors, and native SSR login/session behavior
- UI Overhaul 2.0 detour work: approved shell navigation posture, access-denied redirect plus transient feedback, sidebar footer account/scope/theme context, queue-first home posture, explorer-oriented organization work, section-oriented form-builder UI, and section description/column-count persistence

### Current UI baseline

The application shell already exposes meaningful user-testable surfaces:

- role-aware login and shared home entry
- product-area navigation for Home, Organization, Forms, Workflows, Responses, Components, Dashboards, and admin-area Datasets, Administration, and Migration
- dedicated list/detail/create/edit flows for major top-level entities
- dedicated administration list/detail/create/edit/access flows for users and roles
- visible separation between product-facing and internal/operator areas
- Components and Datasets are native internal inspection surfaces
- Report, aggregation, and chart APIs remain compatibility implementation details, but reports are no longer mounted as a primary UI route or forward planning model

### Current implementation gaps

The main gaps are no longer raw backend feasibility. The remaining work is to:

- finish backend decomposition for workflow, response, reporting, dashboard, and dataset behavior so new work does not compound large route modules
- complete response/runtime, dataset, component, and dashboard authoring in end-user-facing application shape
- retire or clearly isolate remaining transitional reporting compatibility APIs
- transition deprecated analytical endpoints into adapter-only paths behind the target `Dataset -> Component -> Dashboard` model
- close scoped analytical visibility gaps as dataset, component, and dashboard execution surfaces become real application features
- maintain a green dependency-audit posture and document any accepted advisory exceptions

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

The forward model is:

```text
Capture -> Runtime -> Materialization -> Dataset -> Component -> Dashboard
```

More specifically:

```text
Forms/Workflows -> Responses -> Materialized Sources -> DatasetRevision -> ComponentVersion -> Dashboard
```

This roadmap plans the transition toward:

- `Dataset` and immutable `DatasetRevision`
- `Component` and versioned `ComponentVersion`
- mutable `Dashboard` composed from `ComponentVersion`
- printable reports as a later separate artifact, not a core v1 analytical asset

## Approved Carry-Forward Backlog

The following items were accepted during Sprint 1A and Sprint 1B review and should be treated as scheduled follow-up work rather than open-ended notes.

Sprint 1C mandatory acceptance points (must be present to close Sprint 1C):

- Scope-aware naming: when the highest assigned scope is `Partner`, the primary organization list shows `Partner List` instead of `Organization List`.
- Hierarchy navigation: replace the flat organization card layout with a fuller-width tree navigator so scoped users can browse the tree structure directly.
- New User Screen: add an in-app `/administration/users/new` flow so admins can create users without direct API calls, including email, display name, password, active status, initial roles, and a clear follow-up path to scope/delegation access assignment.

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

- route-level code splitting for heavy operator routes, starting with `/migration`
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

### Sprint 1E: Form Semantic Versioning And Compatibility Automation

**Outcome:** form publishing automatically assigns semantic version and major-version compatibility without asking users for manual version labels or compatibility-group selection.

**Build:**

- publish-time server-side semantic version derivation for form versions
- structural compatibility classification at publish time
- automatic major-version reuse for compatible revisions and automatic major-version rollover for breaking revisions
- publish-time diff summary that explains whether the revision is `PATCH`, `MINOR`, or `MAJOR`
- automatic binding of dataset and direct report consumers to the current published form major so existing consumers do not drift across breaking revisions
- explicit handling for direct form-bound reports and datasets so breaking form changes surface stale-dependency warnings without requiring users to interpret compatibility identifiers manually

**Application UI delivered this sprint:**

- draft version flows that defer semantic version and major-version assignment until publish
- publish review screens that show the proposed semantic version, major-version decision, and downstream impact before confirmation
- compatibility status messaging on form detail and edit routes so authors can see when a published revision stayed in the current major line or started a new one

**User-testable exit condition:** a tester can revise a draft form version, publish it, receive an automatically assigned semantic version and major-version compatibility outcome at publish time, and verify from the UI whether the revision stayed in the same major line or created a new one without entering version labels or compatibility-group identifiers manually.

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

- table, queue, picker, and detail-readout surfaces now have approved Rust/UI-style direction in `docs/ui-table-inventory.md`
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

### Sprint 3B: Dataset Advanced Authoring Slice (Next)

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

### Sprint 3C: Dataset Revision And Compatibility Slice

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

### Sprint 4A: Table Component Slice

**Outcome:** table-oriented presentation assets become first-class components.

**Build:**

- `DetailTable` and `AggregateTable` authoring
- component versioning and publication
- validation and dataset-revision binding behavior
- any retained legacy analytical endpoints stay adapter-only; no new core behavior may deepen deprecated asset families
- touched reporting and component routes continuing hybrid-shell removal rather than creating a second long-lived bridge
- component list/detail endpoints enforce scoped dataset and component visibility with negative operator coverage

**Application UI delivered this sprint:**

- component directory/detail/create/edit/publish flows
- table viewers inside the application

**User-testable exit condition:** a tester can create, version, publish, and view table components in the app.

### Sprint 4B: Chart And Stat Component Slice

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

## Phase 5: Dashboards, Scoped Analytics, And Dependency Upgrade UX

### Sprint 5A: Dashboard Composition Slice

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

### Sprint 5B: Scoped Analytics And Presentation Hardening Slice

**Outcome:** dataset, component, and dashboard execution is scope-safe now that those application surfaces exist.

**Build:**

- enforce explicit scoped restriction rules for dataset previews, component execution, and dashboard viewing when such rules are authored
- apply scoped metadata visibility to datasets, dataset revisions, component versions, dashboard composition, and linked presentation assets
- add negative regression coverage proving scoped operators cannot see rows, linked entities, component metadata, or dashboard contents blocked by explicit visibility or restriction rules
- keep any retained deprecated analytical endpoints adapter-only and align their authorization with the canonical dataset, component, and dashboard contracts when touched
- move touched dataset, component, and dashboard execution paths toward bounded-context service and repository boundaries
- preserve clear empty and forbidden states when explicit visibility or restriction rules remove rows or linked assets

**Application UI delivered this sprint:**

- existing Dataset, Component, and Dashboard surfaces remain usable with corrected scoped behavior
- operators receive understandable scoped empty states rather than leaked metadata or raw authorization failures

**User-testable exit condition:** a scoped operator can preview datasets, run/view components, and view dashboards according to authored visibility and restriction rules, while an admin still sees the full seeded analytical set.

### Sprint 5C: Upgrade And Stale Dependency Slice

**Outcome:** stale dependency and rebind flows are usable.

**Build:**

- warning/blocking findings
- carry-forward and rebinding flows
- publication guards for incompatible changes
- stale dependency, carry-forward, and rebinding flows operating on typed dataset, component, and dashboard relationships
- publication guards consuming the typed compatibility and dependency outputs introduced in Sprint 3C

**Application UI delivered this sprint:**

- dependency health and upgrade flows in the application

**User-testable exit condition:** a tester can update dependent assets and resolve or defer findings through UI.

## Phase 6: Migration, Hardening, And Pilot Readiness

### Sprint 6A: Migration And Legacy Mapping Slice

**Outcome:** migration and import flows align to datasets, components, and dashboards.

**Build:**

- mapping docs and verification flows aligned to the new model
- migration UI references into canonical product/detail screens
- updated operator verification paths
- an explicit inventory of all remaining hybrid-shell and legacy-builder surfaces plus a deletion plan for each
- migration and operator screens pointing to canonical native application routes wherever replacements exist
- reconcile docs archive references so canonical docs do not link to absent `docs/archive` sources, or restore the archived sources if they are intended to remain part of the repo

**Application UI delivered this sprint:**

- coherent migration/operator screens tied to the new model

**User-testable exit condition:** operators can validate migration outcomes using real application surfaces.

### Sprint 6B: Pilot Hardening Slice

**Outcome:** the app is stable for broader testing.

**Build:**

- end-to-end coverage
- smoke-path coverage
- permission hardening
- performance cleanup
- explicit unsupported-v1 documentation
- keep the roadmap closed to any reintroduction of the hybrid shell
- complete a final permission and session audit, stable error-envelope cleanup, and performance hardening
- replace permissive production CORS with environment-specific/same-origin policy suitable for cookie sessions
- verify browser login no longer exposes bearer tokens except through explicit script/test/API token flows
- verify that no primary route depends on HTML-string route shells, `/bridge/*`, or retained legacy bridge assets

**Application UI delivered this sprint:**

- no new primary surface, but all existing slices remain coherent and testable

**User-testable exit condition:** the application remains fully testable through intended UI flows after hardening.

## Deferred Beyond This Roadmap

- printable report artifacts composed from prose and components
- full visual dashboard designer beyond the required composition flows
- fuzzy joins, complex window functions, and other analytical features not required for v1
- broader home-surface specialization after the shared shell and role-ready flows are stable
