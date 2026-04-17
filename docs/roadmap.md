# Tessara Roadmap

This roadmap is authoritative as of April 15, 2026. It starts from the current implemented baseline, identifies the transition from the current reporting stack to the re-aligned target model, and defines future delivery as explicit vertical-slice sprints.

## Delivery Rule

Every future sprint is a full vertical slice.

- Every sprint must deliver both underlying functionality and usable application UI.
- The application must remain in a user-testable condition in the intended end-user-facing shape after each sprint.
- Backend-only completion does not satisfy roadmap completion.
- Internal/admin/configuration screens may evolve inside the same sprint, but they do not replace the requirement for coherent application UI.
- Any sprint that touches existing route or UI surfaces must migrate those touched surfaces onto the native Leptos SSR platform in that same sprint.
- Touched surfaces do not count as complete if they still depend on the hybrid shell pattern through `application.rs` HTML-string shells, `inner_html` route injection, or `app-legacy.js`.
- Every sprint must reduce the remaining hybrid-shell footprint, with the explicit end-state that the hybrid shell is fully gone when the roadmap is complete.

## Sprint completion protocol (applies to every sprint)

- Run a local deployment refresh with:
  - `.\scripts\local-launch.ps1` for standard updates, or `.\scripts\local-launch.ps1 -FreshData` when the local UAT dataset should be reseeded from scratch.
- Print and run the sprint UAT script:
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Confirm the UAT script output includes current route ownership and role-gated behavior before closing the sprint.
- Confirm every route surface touched in the sprint runs through native SSR ownership rather than the retained hybrid shell before closing the sprint.

## Current State Of Development

### Implemented baseline

The codebase already includes a substantial vertical foundation:

- local Docker-based development, runnable Rust service, seeded demo workflows, and smoke helpers
- explicit login flow, session handling, capability-derived UI access profiles, role-aware navigation, and route guards
- application-grade administration screens for users, roles, scoped access, and delegations
- admin-managed role creation and assignment with capability bundles
- configurable organization hierarchy and metadata-backed nodes
- form, form version, section, and field support with publish lifecycle
- draft/save/submit response flows and review behavior
- reporting/storage slices for datasets, reports, aggregations, charts, and dashboards
- legacy fixture validation, dry-run, import rehearsal, and demo seed paths
- a Leptos SSR shell with focused routes under `/app`, `/app/admin`, `/app/reports`, and `/app/migration`

### Current UI baseline

The application shell already exposes meaningful user-testable surfaces:

- role-aware login and shared home entry
- product-area navigation for Home, Organization, Forms, Responses, Reports, Dashboards, Administration, and Migration
- dedicated list/detail/create/edit flows for major top-level entities
- dedicated administration list/detail/create/edit/access flows for users and roles
- visible separation between product-facing and internal/operator areas, though the separation is still incomplete

### Current implementation gaps

The main gaps are no longer raw backend feasibility. The remaining work is to:

- deepen organization, form, field, and workflow authoring so they no longer depend on mixed builder-era behavior
- complete response/runtime, dataset, and dashboard authoring in end-user-facing application shape
- transition the reporting stack from the current `Report/Aggregation/Chart` model to the target `Component` model

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

## Frontend Platform Foundation

Before deeper application-surface replacement continues, the frontend platform should follow this explicit sequence.

### Platform Sprint A: Cargo-Leptos Foundation

**Outcome:** the UI runs through a real `cargo-leptos` build pipeline while keeping the current single-binary deployment shape.

**Build:**

- multi-package `cargo-leptos` workspace metadata with `tessara-api` as the server binary and `tessara-web` as the frontend library
- built wasm/js package served by the existing `axum` binary
- shared stylesheet emitted through the `cargo-leptos` pipeline
- hydrated Leptos router preserving the current route surface
- transitional bridge scripts isolated from Rust string literals into explicit frontend assets

**Application UI delivered this sprint:**

- preserved existing routes remain user-testable
- the app shell and current route bodies still render while the runtime/build contract moves under them

**Bridge surfaces still expected after this sprint:**

- admin workbench
- current product/internal route bodies backed by the retained bridge controller

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

- route-level code splitting for heavy operator routes, starting with `/app/migration`
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

- establish `tessara-ui` as the shared internal component library for Tessara
- use `ui-guidance.md` and `docs/style-examples/` as the specification source for component appearance and behavior
- extract the first stable primitives for shared page headers, action groups, cards, panels, metadata strips, inputs, field wrappers, and table or list toolbar patterns
- move touched route surfaces onto `tessara-ui` incrementally while keeping the shared shell stable and SSR-first
- stop adding new bespoke route-level UI patterns when an approved component spec already exists

**Application UI delivered this sprint:**

- current shared routes begin rendering through a recognizable `tessara-ui` visual system instead of route-by-route markup drift
- new Sprint 2A assignment and response-start work can land on top of shared component primitives rather than introducing another parallel styling layer

**User-testable exit condition:** a tester can move through the current shared application surfaces and see consistent headers, actions, cards, panels, and common control styling, and engineers can extend the same component layer for the next workflow-runtime sprint without inventing a new surface pattern each time.

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

### Sprint 2B: Draft, Submit, And Review Response Slice (Next)

**Outcome:** the end-user response lifecycle is coherent and complete.

**Build:**

- pending, draft, submitted, and read-only review flows
- strict submit behavior
- canonical response persistence surfaced through application flows

**Application UI delivered this sprint:**

- polished Responses area aligned to the intended end-user-facing experience

**User-testable exit condition:** a tester can save draft, resume, submit, and review responses through the application UI.

### Sprint 2C: Multi-Step Workflow Authoring And Execution

**Outcome:** workflows are no longer limited to a single response step, and runtime execution can advance across explicit step definitions.

**Build:**

- multi-step workflow version authoring with ordered step definitions
- explicit step transitions and runtime progression across workflow instances
- assignment support for step-specific work rather than only workflow-level single-step work
- response handoff behavior between steps, including completion of one step and activation of the next
- publish-time validation that multi-step workflow versions are structurally complete

**Application UI delivered this sprint:**

- workflow authoring screens that let operators define and inspect multi-step workflow versions
- runtime surfaces that show current step, upcoming step, and completed-step history for in-flight work

**User-testable exit condition:** a tester can create a workflow with more than one step, assign and start it, complete the first step, and observe the next step become the active work item through the application UI.

### Sprint 2D: Runtime Status And Materialization Slice

**Outcome:** runtime execution and materialization readiness are visible and usable.

**Build:**

- workflow/runtime status visibility
- materialization readiness and refresh status
- operator-facing monitoring screens

**Application UI delivered this sprint:**

- coherent internal runtime/materialization surfaces that do not disrupt the main user shell

**User-testable exit condition:** operators can inspect runtime and readiness through the app while end-user flows remain working.

## Phase 3: Dataset Engine And Revisions

### Sprint 3A: Dataset Authoring Slice

**Outcome:** datasets become first-class application assets.

**Build:**

- dataset directory/detail/create/edit flows
- source composition, row filters, calculated fields, and previews
- clearer separation between authoring and viewing surfaces

**Application UI delivered this sprint:**

- usable dataset authoring screens in the application

**User-testable exit condition:** a tester can create, inspect, and edit datasets through app UI.

### Sprint 3B: Dataset Revision And Compatibility Slice

**Outcome:** revision behavior is visible and manageable.

**Build:**

- revision publishing and revision history
- compatibility findings
- carry-forward behavior
- dependency visibility

**Application UI delivered this sprint:**

- revision history, detail, and compatibility screens

**User-testable exit condition:** a tester can revise a dataset and understand downstream impact from the UI.

## Phase 4: Components

### Sprint 4A: Table Component Slice

**Outcome:** table-oriented presentation assets replace old report/aggregation planning.

**Build:**

- `DetailTable` and `AggregateTable` authoring
- component versioning and publication
- validation and dataset-revision binding behavior

**Application UI delivered this sprint:**

- component directory/detail/create/edit/publish flows
- table viewers inside the application

**User-testable exit condition:** a tester can create, version, publish, and view table components in the app.

### Sprint 4B: Chart And Stat Component Slice

**Outcome:** visual presentation assets are first-class components.

**Build:**

- `Bar`, `Line`, `Pie/Donut`, and `StatCard` authoring
- component-specific validation and viewing behavior

**Application UI delivered this sprint:**

- visual component builder and viewer screens

**User-testable exit condition:** a tester can build and view visual components without old chart/aggregation-specific workbench flows.

## Phase 5: Dashboards And Dependency Upgrade UX

### Sprint 5A: Dashboard Composition Slice

**Outcome:** dashboards compose component versions through application-grade flows.

**Build:**

- dashboard directory/detail/create/edit/view flows
- component placement and composition
- clearer product-facing dashboard viewers

**Application UI delivered this sprint:**

- readable product-facing dashboard screens
- usable internal dashboard composition screens

**User-testable exit condition:** a tester can assemble and view dashboards through the app.

### Sprint 5B: Upgrade And Stale Dependency Slice

**Outcome:** stale dependency and rebind flows are usable.

**Build:**

- warning/blocking findings
- carry-forward and rebinding flows
- publication guards for incompatible changes

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

**Application UI delivered this sprint:**

- no new primary surface, but all existing slices remain coherent and testable

**User-testable exit condition:** the application remains fully testable through intended UI flows after hardening.

## Deferred Beyond This Roadmap

- printable report artifacts composed from prose and components
- full visual dashboard designer beyond the required composition flows
- fuzzy joins, complex window functions, and other analytical features not required for v1
- broader home-surface specialization after the shared shell and role-ready flows are stable
