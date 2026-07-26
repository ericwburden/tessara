# Sprint 6C Plan: Independently Deployed Dashboard Module Slice

Status: kicked off on 2026-07-25 from clean `main` commit
`c4e291c32645af65773726ab6a93449f4bef2c4a`.

- Branch: `codex/sprint-6c`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6c`
- Roadmap source:
  `Sprint 6C: Independently Deployed Dashboard Module Slice (Next)`
- Predecessor: `Sprint 6B2: Secure Module Operation Slice (Complete)`

## Sprint Summary

Sprint 6C extracts the existing Dashboard product area into Tessara's first
independently deployed existing feature module. Dashboard authoring, viewing,
API behavior, persistence, migrations, configuration, health, readiness, and
diagnostics move behind one Dashboard Module Instance and its isolated
database. The established Sprint 5A directory, editor, detail, and viewer
experience remains available through the normal same-origin shell.

Core remains authoritative for installation identity, actors, Organization
scope, authorization revisions, Module Releases and Instances, navigation, and
the temporary in-process Components provider. Dashboard owns composition,
placement, visibility, and its stored typed ComponentVersion references.
Dashboard never reads Core or Components tables directly and never receives a
browser cookie, Core credential, or reusable Core authority.

The existing `TypedResourceReference` and `ResourceResolutionV1` contracts are
the platform vocabulary for the transition Components binding. Sprint 6C adds
the Dashboard-specific first-party compatibility contract and action surface
without redefining Component lifecycle semantics or presenting the in-process
Components contribution as a Module Instance.

## Sprint Specifications

### Dashboard Module Boundary

- Add an independently runnable Dashboard service with module-owned API and
  native SSR routes for directory, creation, detail, editing, and viewing.
- Move Dashboard orchestration, persistence, migrations, configuration,
  readiness, health, and sanitized diagnostics out of `tessara-api`.
- Reuse the shared module SDK and `ShellContextV1` validation so Dashboard
  renders a complete Tessara document without receiving Core browser state.
- Advertise Dashboard routes, navigation, contracts, resource types,
  configuration, diagnostics, and `dashboards:read` / `dashboards:manage`
  security capabilities in one authoritative module manifest.
- Keep Core responsible only for same-origin gatewaying, shell projection,
  authorization exchange, Module Management, and desired
  configuration/enablement orchestration.

### Isolated Persistence And Seed

- Add a Dashboard-owned baseline migration and one logical database per
  Dashboard Module Instance with distinct owner, migration, and runtime roles.
- Move `dashboards`, visibility, placement, idempotency, and module
  configuration state into the Dashboard database.
- Store only installation-scoped typed references to ComponentVersion
  resources; remove Dashboard runtime joins or foreign keys into Core or
  Components persistence.
- Restructure the development seed directly for the split database layout.
  There is no production-data compatibility obligation, but the resulting
  seed must be deterministic and idempotent.
- Remove Dashboard tables from the final Core baseline after the extraction is
  proven, then apply both squashed baselines transactionally to disposable
  empty databases.

### Transition Components Compatibility Contract

- Define a versioned, first-party Core Release contract for resolving the
  transition-only `core_installation`-owned ComponentVersion resource type.
- Bind the contract to explicit metadata and render/execute actions. New
  external Blueprints cannot select this transition binding.
- Resolve through Core's temporary Components adapter using
  `TypedResourceReference`, `ResourceResolutionV1`, and Core-issued downstream
  authorization. Never infer authority from a reference.
- Keep the original actor, Dashboard presenting service, installation,
  audience, declared dependency/contract/action, capability-to-scope bindings,
  revisions, expiry, and replay semantics intact across the hop.
- Return a stable restricted projection before resolving resource existence
  for unauthorized or not-evaluated callers.
- After authorization, distinguish provider unavailable, incompatible
  contract, inactive, superseded, provider-resource tombstoned,
  owner-module-instance tombstoned/data-destroyed, missing, and not-evaluated
  outcomes without changing provider-owned lifecycle meaning.
- Record the transition binding and the Sprint 8A explicit migration
  requirement in the manifest and contract documentation.

### Product And UI Continuity

- Preserve the Sprint 5A Dashboard directory, create, detail, editor, and
  viewer URLs and behavior through same-origin routing.
- Preserve the fixed 12-column / 240-row grid, 240-placement limit,
  component-kind geometry rules, visibility scoping, idempotent composition
  reconciliation, responsive behavior, and no-JavaScript usefulness.
- Project Component metadata and render results through the compatibility
  adapter rather than copying Components product state into Dashboard.
- Add clear, contained placement treatments for restricted, unavailable,
  inactive, superseded, tombstoned, destroyed-owner-data, missing,
  incompatible, and not-evaluated states.
- Add Dashboard configuration and diagnostics panels under Core Module
  Management using the established independently deployed module patterns.

### Deployment And Operations

- Add `deploy/sprint-6c/compose.yaml` with Core, Traefik, PostgreSQL,
  installation control, Scoped Records where retained by the reference stack,
  and Dashboard services. Only approved same-origin routes are public.
- Add a repeatable `scripts/bootstrap-sprint-6c-deployment.ps1` that
  materializes the Dashboard release/instance/receipt and seed, and proves a
  second identical invocation is a no-op.
- Build release images with exact source commit, source tree, dirty state, and
  release profile labels, then verify the running immutable image before
  retaining deployment, smoke, UAT, or browser evidence.
- Keep Dashboard failure contained: Core Module Management remains reachable,
  Dashboard routes show the approved unavailable state, and unrelated routes
  continue to operate.

## Acceptance Criteria

1. Dashboard runs as a separately deployed service with its own database,
   migration/runtime identities, module manifest, configuration, health,
   readiness, diagnostics, API, and native SSR UI.
2. Dashboard runtime credentials cannot read Core, Components, Scoped Records,
   deployment-control, or another Module Instance database.
3. Stored placements use installation-scoped, `core_installation`-owned typed
   ComponentVersion references and contain no relational dependency on Core
   Component tables.
4. The first-party compatibility adapter is versioned, action-bound,
   transition-only, unavailable to new external Blueprints, and explicitly
   marked for Sprint 8A migration.
5. Core-issued downstream grants bind the original actor, Dashboard service,
   installation, target audience, declared contract/action, independent
   capability scopes, revisions, expiry, and replay policy.
6. Unauthorized and not-evaluated resolution is non-disclosing; authorized
   resolution distinguishes every roadmap-required availability, lifecycle,
   identity, compatibility, and owner-data state.
7. Dashboard directory, create, detail, editor, and viewer preserve the Sprint
   5A experience through the normal shell and same-origin URLs.
8. Dashboard configuration and diagnostics are reachable from Core Module
   Management without giving Core Dashboard product-data access.
9. Dashboard or Components outages remain contained and produce clear
   Dashboard/placement states while Core and unrelated routes remain usable.
10. Fresh seed/bootstrap is deterministic and idempotent; final Core and
    Dashboard baseline migrations apply from scratch with the expected
    one-row ledgers.
11. Contract, permission, outage, database-isolation, API, SSR, responsive,
    smoke, UAT, and Playwright coverage pass against the exact clean source
    commit used to build the retained stack.

## Manual Test Plan

1. Launch a fresh Sprint 6C stack, sign in as the seeded administrator, open
   Dashboards from the normal navigation, and exercise directory, create,
   detail, editor, save, and viewer flows.
2. Verify Dashboard appears as an independently deployed Module Instance in
   Module Management with configuration, release, health, readiness,
   diagnostics, routes, capabilities, and navigation declarations.
3. Use a non-admin actor with disjoint Dashboard and Component scope. Confirm
   only authorized Dashboards and placements resolve and that restricted
   references do not disclose whether a known or random ComponentVersion
   exists.
4. Stop Dashboard. Confirm its routes show the contained Core-owned fallback,
   Module Management remains usable, and unrelated product routes continue to
   work. Restore Dashboard and verify its identity and data.
5. Stop or degrade the temporary Components provider. Confirm each affected
   placement shows the provider state without making the whole Dashboard or
   Core unavailable.
6. Exercise inactive, superseded, provider-resource tombstoned,
   owner-tombstoned/data-destroyed, missing, incompatible, and not-evaluated
   fixtures and confirm their visible treatments after authorization.
7. Inspect the Dashboard database and credentials. Confirm Dashboard rows and
   placements live only there and negative cross-database access fails.
8. Run the deployment bootstrap twice and confirm the second invocation is a
   no-op with the same Dashboard Module Instance, database binding, and
   receipt revision.
9. Inspect directory, editor, viewer, configuration, diagnostics, and degraded
   states at 1280, 768, and 390 pixels, in both themes, with keyboard-only and
   no-JavaScript checks where applicable.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-module-contract`
- `cargo test -p tessara-dashboards`
- `cargo test -p tessara-web-dashboards`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- Dashboard service unit and disposable-PostgreSQL integration tests for
  schema, repository, API, native SSR, configuration, health, diagnostics,
  idempotency, authorization, and outage projection.
- Core compatibility-adapter tests for declared caller/audience/action,
  transition-only binding, reference ownership/type, grant exchange,
  revision invalidation, nondisclosure, and all detailed resolution states.
- Database-isolation tests using Dashboard runtime credentials against every
  non-Dashboard database.
- `docker compose -f deploy\sprint-6c\compose.yaml config --quiet`
- `.\scripts\bootstrap-sprint-6c-deployment.ps1` twice.
- `.\scripts\local-launch.ps1` remains in the checklist but is expected to be
  replaced for retained 6C evidence by the documented Sprint 6C destructive
  reset/build/up/bootstrap cycle because the root profile is not the
  independent Dashboard topology.
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `npm --prefix .\end2end test`
- `.\scripts\validate-e2e.ps1 ...` against the exact Sprint 6C deployment
  evidence and `artifacts/sprint-6c-closeout/`.

## Closeout-Readiness Plan

- Deployment profile: `deploy/sprint-6c/compose.yaml`.
- Idempotent materialization:
  `.\scripts\bootstrap-sprint-6c-deployment.ps1`.
- Source provenance: release images and deployment evidence must record exact
  commit, tree, dirty state, release profile, and immutable image digest.
- Migration checkpoint: after the last schema change, squash development
  migrations into Core and Dashboard baselines, apply each to disposable empty
  databases transactionally, and record exact ledger contents and baseline
  SHA-256 digests before the final source-exact build.
- Retained evidence directory: `artifacts/sprint-6c-closeout/`, containing at
  least `deployment-fresh.json`, `smoke-fresh.json`, `uat-fresh.json`,
  `e2e-fresh.json`, `e2e-fresh.summary.json`, and supported hash sidecars.
- Harness reconciliation: update smoke, UAT, Playwright manifests/tests,
  seeded actors, route/navigation/module inventories, capability expectations,
  and database assumptions in the same change that alters them. Prefer
  semantic inventory assertions over copied exact counts.
- Non-admin proof: one actor has disjoint Dashboard and Component authority
  over different Organization subtrees; known and random unauthorized
  ComponentVersion references must have the same public result.
- Clause mapping:
  - Separate process/database: manual tests 1, 2, and 7; Compose, migration,
    integration, and negative credential tests.
  - Typed transition reference/adapter: manual tests 3 and 6; contract and
    compatibility-adapter tests.
  - Downstream scoped authorization: manual test 3; grant exchange,
    revision, audience/action, replay, and nondisclosure tests.
  - Manifest/configuration/diagnostics: manual test 2; manifest, Module
    Management, configuration, health, and diagnostics tests.
  - Preserved authoring/viewing: manual test 1; API/web and canonical
    Playwright Dashboard suites.
  - Contained provider/module failure: manual tests 4–6; outage and browser
    degradation assertions.
  - Fresh data/bootstrap: manual tests 7–8; baseline, seed, ledger, deployment,
    smoke, UAT, and retained-evidence assertions.

## Ordered Implementation Plan

1. Freeze the extraction inventory: Dashboard routes, API DTOs, tables,
   migrations, seeds, capabilities, UI bootstraps, tests, and direct
   Component/Core dependencies.
2. Define the Dashboard-specific transition ComponentVersion reference and
   compatibility-contract/action vocabulary on the existing platform
   resource-resolution types.
3. Add the Dashboard service crate, manifest, configuration, health,
   readiness, diagnostics, and native shell bootstrap.
4. Add the isolated Dashboard baseline, roles, repository, deterministic seed,
   and data-isolation tests.
5. Move Dashboard API/service/reconciliation behavior behind the Dashboard
   process boundary while preserving DTO and idempotency behavior.
6. Implement the Core Components compatibility adapter and secure downstream
   grant exchange, including every required resolution state.
7. Move existing Dashboard UI routes into module-owned SSR/hydration and
   connect them through Core same-origin routing.
8. Add Core Module Management configuration/diagnostics integration and
   contained unavailable states.
9. Add Sprint 6C Compose, bootstrap, provenance, smoke, UAT, and browser
   harness changes alongside the new inventory.
10. Run the closeout-readiness audit, squash/apply baselines, commit the clean
    implementation source, and perform one retained source-exact cycle.

## Dependencies And Blockers

- Sprint 6B2's `ShellContextV1`, signed authorization grants, typed resource
  references, resource-resolution envelope, module SDK, Module Management, and
  Compose topology are required foundations and are complete.
- Existing `tessara-dashboards`, `tessara-web-dashboards`, Dashboard API
  modules, Sprint 5A browser coverage, and historical capacity fixture are the
  behavior baseline.
- The in-process Components provider remains intentionally transitional. The
  extraction must not broaden it into a general external-module contract or
  pre-implement Sprint 8A.
- Production third-party verification, Blueprint automation, general
  Supervisor lifecycle UX, and physical Components extraction are later
  roadmap scope.
- No kickoff blocker is currently known. Any discovery that requires changing
  Dashboard product behavior rather than relocating it must be recorded and
  resolved before implementation expands.
