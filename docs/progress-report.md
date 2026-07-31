# Progress Report

Historical implementation log. Older entries may mention deleted planning
artifacts, old sprint worktrees, `/app/*` routes, or earlier crate names. Use
`docs/roadmap.md`, `docs/architecture.md`, and `docs/README.md` for current
project direction.

“Next Sprint” labels inside dated entries are historical snapshots and may be
superseded. Use the current sequencing in `docs/roadmap.md`.

## 2026-07-31 - Sprint 6E Implementation

- Extracted the Dashboard domain, complete documents, immutable JS/WASM
  hydration bundle, and provider-owned Components contract from root web/Core
  source ownership.
- Replaced the Dashboard-specific Core gateway with manifest-driven document,
  public API, asset, authorization, security, and Organization projection
  routing.
- Produced the source-exact Dashboard `2.0.0` baseline at `7b3e0341` and the
  Dashboard-only `2.0.1` candidate at `69d19dae`.
- Upgraded the retained deployment to receipt revision 3, switched a healthy
  `2.0.1` candidate without restarting the baseline Dashboard container, and
  restored the active route to healthy `2.0.0`.
- The Sprint 6E stack remains running on the baseline slot; the candidate
  container is stopped and its image is retained for the next verification
  pass.

## 2026-07-31 - Sprint 6E Planning Kickoff

- Sprint: `Sprint 6E: Dashboard SDK Adoption And Source Independence Slice`,
  selected from the sole roadmap heading marked `(Next)`.
- Status: plan approved after review; implementation began from clean
  post-Sprint-6D `main` commit
  `3f2acf6a7151fa59983bd2fab42123db65b804aa`.
- Branch: `codex/sprint-6e`.
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6e`.
- Plan: `docs/sprints/sprint-6e-plan.md`.
- Planned verification: formatting; targeted Dashboard/canonical SDK tests;
  native/WASM package and source audits; Dashboard-owned SSR/hydration/assets;
  API/web regressions; Sprint 6E Compose and idempotent bootstrap;
  Dashboard-only upgrade/rollback evidence; smoke; UAT; direct and retained
  Playwright.
- Approved decisions: Dashboard-owned `tessara-dashboard-ui`; permanent
  provider-owned `tessara-components-contract`; retained route-free Component
  viewer; manifest-driven generic routing; private-gateway blue/green release
  switch; clean `2.0.0` baseline and Dashboard-only `2.0.1` candidate commits.

## 2026-07-31 - Sprint 6D Source-Exact Closeout

### Completed

- Sprint 6D is complete. Canonical `tessara-module-contract`,
  `tessara-module-runtime`, `tessara-module-ui`, and
  `tessara-module-testkit` packages now own the policy-neutral module
  contract, startup/verification/operations, complete-document UI/assets, and
  reusable conformance behavior.
- The non-product `tessara.reference.module-sdk` release proves independent
  native SSR, WASM hydration, configuration/security persistence, probes,
  diagnostics, immutable assets, authorization nondisclosure, outage
  containment, and shutdown without linking root `tessara-web` or
  `tessara-api`.
- Scoped Records consumes the canonical runtime and UI while retaining its
  product and authorization semantics. Dashboard behavior remains unchanged;
  its intentional root-`tessara-web` source edge is recorded, not allowlisted,
  and handed to Sprint 6E.
- The final source-exact implementation is commit
  `e313e1a9f7c412c8d4651af8d19e6178c446a696`, tree
  `84544593ce21d4bf9405bbbfe45f5d1f275eeb66`.
- The first candidate evidence cycle found that container SIGTERM did not
  reach the canonical shutdown future. The candidate evidence was discarded,
  Unix SIGTERM handling was added in the implementation commit above, and the
  stack and complete evidence set were recreated. No test expectation changed
  during the correction.
- All 18 required retained artifacts and their SHA-256 sidecars are under
  `artifacts/sprint-6d-closeout/`.

### Validation

- A destructive fresh reset, release-image build, four database migrators,
  first bootstrap, and exact second-bootstrap no-op passed. All five Tessara
  images are labeled with the closing implementation commit/tree and clean
  release provenance.
- Native/WASM boundaries, source ownership, exact compatibility inventory,
  and shared reference conformance passed. Canonical packages have zero
  forbidden product/Core dependencies or terminology findings.
- Smoke and Sprint UAT passed. Graceful SIGTERM exited 0 in 0.436 seconds with
  unchanged state; the outage check retained Core health and Dashboard at 200,
  served the Core-owned reference fallback at 503, and restored the reference
  route to 200.
- `cargo test -p tessara-api --locked` passed 156 unit tests and every active
  database integration suite; `cargo test -p tessara-web --locked` passed 80
  tests.
- `scripts/validate.ps1` passed from a full `cargo clean` against four fresh,
  role-specific test databases, including the release timing proof.
- Direct Playwright and the source-bound retained wrapper each passed 62/62.
  There were zero skipped, unexpected, flaky, filtered, or retried retained
  tests. Existing Core, Organization, Forms, Workflows, Responses, Datasets,
  Components, Dashboard, Module Management, responsive, keyboard, theme,
  authorization, SSR, no-JavaScript, and hydration expectations remained
  intact.
- The database-backed Scoped Records regression passed six unit checks and
  its secure integration check. The 20-point manual UAT matrix passed and the
  final evidence digest gate verified all 18 artifacts.

### Next Sprint

Sprint 6E - Dashboard SDK Adoption And Source Independence. It must adopt the
Sprint 6D canonical packages, remove the recorded
`tessara-dashboard-module -> tessara-web` edge, and prove Dashboard-only
release/upgrade/rollback behavior without reopening Sprint 6D product scope.

### Sprint Handoff / Demo Instructions

The source-exact Sprint 6D stack remains running at
`http://127.0.0.1:8080`.

#### Canonical Reference Experience

- Role: enrolled administrator.
- Path: normal Tessara navigation to the reference module route.
- Steps: sign in, open the reference destination, refresh it directly, inspect
  the complete server-rendered document, exercise keyboard focus, switch
  light/dark/system themes, and inspect it at 1280 px, 768 px, and 390 px.
- Expected: the module owns the complete SSR/hydration document and
  content-hashed assets while presenting the same shell, navigation, tokens,
  accessibility, and responsive behavior as Core.
- Acceptance: PASS.
- Evidence: `manual-uat.md`, `reference-conformance.json`,
  `e2e-fresh.json`, and `e2e-fresh.summary.json`.

#### Configuration, Authorization, And Operations

- Roles: administrator and constrained actor.
- Paths: reference product route, module configuration, readiness/liveness,
  and sanitized diagnostics.
- Steps: submit invalid and normalized valid configuration, read persisted
  state, compare authorized and unauthorized Organization probes, disable and
  re-enable the module, and inspect operational state.
- Expected: stable validation findings, persisted normalized configuration,
  capability/scope enforcement, indistinguishable known/random unauthorized
  results, separate lifecycle probes, and no secret-bearing diagnostics.
- Acceptance: PASS.
- Evidence: `manual-uat.md`, `reference-conformance.json`,
  `uat-fresh.json`, and `compatibility-inventory.json`.

#### Outage, Recovery, And Retained Products

- Role: operator, then enrolled administrator.
- Paths: reference route, `/health`, Dashboard, Scoped Records, and existing
  Core product routes.
- Steps: stop the reference service with SIGTERM, visit its same-origin route,
  confirm unrelated routes, restart it, and repeat the retained product flows.
- Expected: bounded exit with retained state, Core-owned 503 reference
  fallback, uninterrupted Core/Dashboard behavior, healthy recovery, and no
  existing-screen regression.
- Acceptance: PASS.
- Evidence: `shutdown.json`, `outage-recovery.json`,
  `scoped-records-regression.json`, `smoke-fresh.json`, `uat-fresh.json`, and
  `e2e-fresh.json`.

### Acceptance Mapping

| Roadmap or exit clause | Manual proof | Automated proof | Result |
| --- | --- | --- | --- |
| Assign one owner to shared and duplicated behavior | Review ownership rows and the visible Sprint 6E Dashboard finding | Ownership and duplicate-source scan | PASS |
| Establish contract/runtime/UI/testkit boundaries | Inspect manifests, package sources, and native graph | Exact native allowed/forbidden-edge audit | PASS |
| Extract verification, destinations, references, errors, control, probes, diagnostics, tracing, and shutdown without product policy | Exercise reference control/operations and inspect sanitized output | Contract/runtime/testkit/reference suites and source scan | PASS |
| Extract complete-document shell, primitives, tokens, accessibility, assets, and hydration | SSR/no-JavaScript, keyboard, themes, and three-width walkthrough | Native/WASM builds, conformance, and 62-test Playwright suite | PASS |
| Define exact current compatibility and release inventory | Review supported tuple and obsolete rejection | Compatibility inventory gate | PASS |
| Prevent canonical packages reaching Core/root/product implementations | Review native and WASM graph paths | Transitive graph and forbidden-symbol audits | PASS |
| Deliver the non-product reference module and shared testkit | Navigate fixture and inspect conformance output | Six-check shared conformance suite | PASS |
| Document authoring and fast-forward SDK upgrade workflow | Follow the clean-checkout authoring checklist | Markdown links and conformance entrypoint checks | PASS |
| Preserve Core, Dashboard, and Scoped Records; defer Dashboard adoption | Exercise retained product routes and compare existing UI | Rust, smoke, UAT, direct and retained Playwright regressions | PASS |
| Build/run reference without root `tessara-web` or `tessara-api` | Inspect package/image ownership and run reference image | Native/WASM dependency and source audits | PASS |
| Navigate a coherent same-origin complete SSR route | Normal navigation, direct refresh, raw document/assets review | Smoke, UAT, conformance, and Playwright SSR/asset checks | PASS |
| Verify authenticated and unavailable states | Allowed/constrained actors plus disable, stop, and restart walkthrough | Authorization/nondisclosure and outage/recovery checks | PASS |
| Run the shared conformance suite | Inspect all six reported checks | Shared conformance command | PASS |
| Show one canonical implementation per extracted behavior | Review ownership output and explain the Dashboard handoff | Duplicate-source and forbidden-symbol scans | PASS |
| Show module-owned compiled runtime/UI/assets in the image | Inspect image provenance, asset URLs, and cache headers | Image labels/digests and asset assertions | PASS |

## 2026-07-30 - Sprint 6D Implementation

- User approved the complete Sprint 6D test/harness change packet and
  implementation proceeded without compatibility facades or old-manifest
  readers.
- User subsequently approved a forward-only validation cleanup: active
  database tests now use four role-based API fresh/API integration/reference
  module/API enrollment variables, and the pre-production reference module no
  longer carries a legacy-binary rollback compatibility test or dedicated
  upgrade-test database. The workspace-only installation-control fixture now
  likewise uses the role-specific
  `TEST_INSTALLATION_CONTROL_DATABASE_URL` contract.
- Added the canonical contract/runtime/UI/testkit packages and the independent
  non-product reference module with native SSR, WASM hydration, exact manifest
  declarations, JSON configuration/security persistence, probes, sanitized
  diagnostics, immutable assets, and graceful shutdown.
- Deleted `ShellContentV1`, the root Scoped Records fragment bridge, and
  `tessara-web-ui`; moved generic grid geometry to module contract and shared
  primitives/placement mechanics to module UI while retaining product policy
  in Dashboard and Forms.
- Added exact current manifest/platform/package tuples, validated browser
  routes and content-addressed assets, generic Core document/asset routing,
  credential stripping, signed projections, no-store documents, immutable
  assets, and the Core-owned unavailable fallback.
- Scoped Records now consumes canonical runtime verification/startup and
  module UI complete-document rendering. Dashboard behavior remains unchanged
  and its root-web edge remains the explicit Sprint 6E finding.
- Added Sprint 6D Compose/bootstrap scaffolding plus native/WASM source graph
  and exact compatibility-inventory gates.
- Pre-commit validation passed: workspace all-target checking; formatting and
  diff whitespace; 62 contract unit tests plus contract fixture/protocol
  suites; runtime, testkit, 28 module-UI, and four reference-module tests;
  contract/UI/reference WASM checks; the database-backed Scoped Records
  integration test against a disposable clean database; the generic Core
  route matcher; native/WASM source boundaries; exact compatibility
  inventory; Markdown links; Compose configuration; all PowerShell parsers;
  and Playwright discovery of exactly 62 tests in seven files.
- The hardened validation contract subsequently passed its preflight
  self-test, database-free fast gate, canonical full gate against four freshly
  provisioned databases, and serial workspace all-features suite against those
  fixtures plus the installation-control database. The full gate ran both API
  library database proofs; the workspace run also passed installation-control
  and the current reference-module integration test.
- Retained source-exact deployment, smoke, UAT, complete Playwright execution,
  outage/recovery, graceful-shutdown, and manual closeout evidence remains to
  be captured from committed source.

## 2026-07-30 - Sprint 6D Specification Hardening

- Reconciled Sprint 6D with Tessara's pre-production pure fast-forward
  direction: the implementation will support one current manifest and one
  exact Core/protocol/SDK tuple rather than retain development compatibility
  readers, facades, bridges, or previous-minor windows.
- Fixed the canonical crate graph and public provider responsibilities for
  `tessara-module-contract`, `tessara-module-runtime`,
  `tessara-module-ui`, and `tessara-module-testkit`.
- Decided that Sprint 6D moves policy-neutral primitives to module UI,
  deletes `tessara-web-ui`, retains `tessara-web-http` as an independent leaf,
  and deletes `ShellContentV1`. Implementation inventory subsequently refined
  placement ownership: generic geometry is contract-owned, shared DOM
  mechanics are module-UI-owned, and Dashboard/Form policy remains
  product-owned.
- Specified one manifest-driven Core GET/HEAD document and immutable-asset
  proxy, the non-product `tessara.reference.module-sdk` fixture, exact
  authorization/outage/state behavior, and Scoped Records runtime/UI
  adoption.
- Kept Dashboard runtime/UI adoption and root-web removal entirely in Sprint
  6E; the Sprint 6D boundary report must show that debt as nonconforming rather
  than allowlist it.
- Added a clause-by-clause verification/evidence contract. No test, fixture,
  assertion, smoke/UAT/conformance script, Playwright file, or verification
  harness may change until the user approves the consolidated test-change
  packet.
- Production extraction remains blocked until this specification change is
  committed, the test-change packet is approved, and boundary/current-render
  gates are ready to land with their approved tests.

## 2026-07-30 - Sprint 6D Canonical Module SDK And Runtime Kickoff

- Status: kicked off from clean `main` commit
  `89f133f683c1fb1c549b85f57a08098077ac3fba`.
- Branch: `codex/sprint-6d`.
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6d`.
- Plan: `docs/sprints/sprint-6d-plan.md`.
- Planned verification: `cargo fmt --all`, contract/runtime/UI/testkit and
  reference-module targeted suites, native/WASM package-boundary audits,
  `cargo test -p tessara-api`, `cargo test -p tessara-web`, Sprint 6D Compose
  validation and idempotent bootstrap, smoke, UAT, and retained Playwright.
- Immediate focus: inventory current canonical/shared behavior and dependency
  edges, then establish the package/source boundary audits before extracting
  runtime and UI implementations.

## 2026-07-30 - Post-Sprint 6C Module SDK And Extraction Decision

### Decision

- Accept repeated compiled code, container layers, CSS, JavaScript, and WASM
  across Core and module images when every shared behavior has one canonical
  source owner.
- Extract independently versioned, policy-neutral platform contract, module
  runtime, UI SDK/design-system, asset, and conformance packages before
  applying the next feature-module extraction.
- Keep functional behavior owned by exactly one Core area or module and reuse
  it through versioned contracts rather than shared implementation,
  persistence access, or copied source.
- Treat Sprint 6C as the completed Dashboard process/database boundary while
  acknowledging its remaining source/build transition: the Dashboard module
  still links root `tessara-web`, and Core still constructs
  Dashboard-specific web bootstrap types.

### Roadmap Change

- Sprint 6D is now **Canonical Module SDK And Runtime Extraction**.
- Sprint 6E completes Dashboard SDK adoption, route/asset ownership, and
  source/build independence.
- The former Sprint 6D Blueprint and composition work moves intact to Sprint
  6F.
- Components, Datasets, Responses, Workflows, and Forms must use the completed
  Sprint 6D/6E extraction pass, including source-ownership and
  independent-image upgrade/rollback proof.

This sequencing supersedes “Next Sprint” statements in the dated Sprint 6C
closeout entries below without changing their retained implementation or
validation evidence.

## 2026-07-29 - Sprint 6C Final Corrective Closeout

### Completed

- Sprint 6C is complete. Dashboard is independently deployed with its own
  service, database, identities, migration, manifest, configuration,
  diagnostics, APIs, SSR UI, and same-origin product routes.
- Module Management now applies one reusable, schema-driven control pathway to
  Dashboard and Scoped Records. Common configuration, enablement, navigation,
  diagnostics, findings, lifecycle, and route-state behavior has no
  definition-specific Core branch.
- Corrective UAT aligned disabled-state labels and lifecycle assessment,
  removed disabled product navigation, made enablement actionable, repaired
  diagnostics navigation and findings presentation, and retained
  module-specific notes in the final Configuration row.
- Degraded Dashboard placements use the approved warning-tinted tile with one
  icon; the full message, large centered icon, and retry action live in the
  side sheet.
- Final implementation source is commit
  `a5d694f7ef7c68e52a9ac93135846d29d5a061d7`, tree
  `fc5494044be6c8dffa6c38381b5610f49f6619c4`.

### Validation

- Exact-source deployment, bootstrap idempotency, fresh-data evidence, smoke,
  UAT, 61/61 direct Playwright, and 61/61 retained Playwright passed.
- API and web Rust suites, formatting, database isolation, and all ten
  degraded provider states passed. Each degraded state retained all nine
  placements and titles, and the normal enabled/available state was restored.
- Final evidence is
  `artifacts/sprint-6c-final-closeout-2026-07-29-r2/`; command details and all
  twelve acceptance mappings are in
  `docs/sprints/sprint-6c-verification.md`.
- `docs/roadmap.md` was reviewed: Sprint 6C is already marked **Complete** and
  Sprint 6D is already marked **Next**, so no roadmap mutation was required.

### Next Sprint

Sprint 6D - Application Blueprint And Composition Automation Slice.

### Sprint Handoff / Demo Instructions

#### Shared Module Management Template

- Role: administrator (`admin@tessara.local`).
- Paths:
  - `http://localhost:8080/administration/modules`
  - `http://localhost:8080/administration/modules/tessara.dashboards#configuration`
  - `http://localhost:8080/administration/modules/tessara.reference.scoped-records#configuration`
- Steps:
  1. Compare both module Configuration, Overview, Diagnostics, Findings,
     Dependencies, Navigation, and application-state panels.
  2. Edit and save each schema-owned configuration without changing its value.
  3. Disable and re-enable each product route.
- Expected: common structure, controls, status semantics, errors, and
  navigation are identical; only declared metadata, fields, dependencies,
  resources, diagnostics, and product behavior differ. Disabled modules say
  **Disabled**, are not treated as errors, and disappear from product
  navigation while remaining manageable.
- Acceptance check: pass when both modules complete the same administrative
  workflow without a per-module Core screen or adapter.
- Evidence:
  `artifacts/sprint-6c-final-closeout-2026-07-29-r2/playwright-retained-fresh.summary.json`
  and `docs/audits/module-management-consistency-2026-07-27/README.md`.

#### Independent Dashboard Product Flow

- Role: administrator.
- Paths:
  - `http://localhost:8080/dashboards`
  - `http://localhost:8080/dashboards/41933f5c-f02b-47c6-b44f-6edffa32c283`
  - append `/edit` or `/view` for authoring and presentation.
- Steps:
  1. Open the directory and the retained Demo Operations Dashboard.
  2. Review detail, editor, and viewer; select, move, resize, save, and refresh
     a placement without changing its reference.
  3. Confirm nine placements remain and the same Dashboard identity survives.
- Expected: every page remains in the normal Tessara shell at the established
  same-origin URL and preserves Sprint 5A behavior.
- Acceptance check: pass when directory/detail/editor/viewer work after
  refresh and saved composition is stable.
- Evidence:
  `artifacts/sprint-6c-final-closeout-2026-07-29-r2/smoke-fresh.json`,
  `uat-fresh.json`, and `playwright-retained-fresh.json`.

#### Enablement, Outage, And Degraded Placement States

- Role: administrator; use the scripted matrix for state mutation.
- Paths:
  - Dashboard Configuration in Module Management.
  - Retained Dashboard `/edit` and `/view` paths above.
- Steps:
  1. Disable Dashboard and confirm its navigation item disappears while
     Module Management reports a healthy **Disabled** application state.
  2. Re-enable it and run
     `scripts/test-sprint-6c-degraded-states.ps1`.
  3. In the editor, open a warning tile's icon, inspect the centered side-sheet
     icon and full diagnostic, then use **Retry resolution**.
  4. Confirm the script restores the available provider and enabled module.
- Expected: Dashboard or Components failures remain contained, unrelated Core
  routes work, saved titles remain, and no product-data loss occurs.
- Acceptance check: pass when all ten resolution states are distinct, all nine
  placements survive, and the final normal state is restored.
- Evidence:
  `artifacts/sprint-6c-final-closeout-2026-07-29-r2/degraded-states-fresh.json`
  and `deployment-handoff-fresh.json`.

#### Constrained Actor And Reusable Migration Review

- Role: the ephemeral constrained actor from the permissions fixture for the
  access proof; implementer/reviewer for the pathway review.
- Paths:
  - Dashboard API/viewer routes exercised by `end2end/tests/permissions.spec.ts`.
  - `docs/architecture/independent-module-pathway.md`.
- Steps:
  1. Compare known and random ComponentVersion references outside the actor's
     Component scope while Dashboard scope remains authorized.
  2. Confirm both results are nondisclosing and hidden Components never run.
  3. Follow the ownership, manifest, configuration endpoint, deployment,
     bootstrap, and conformance checklist for a hypothetical third module.
- Expected: authorization does not widen across module boundaries, and a third
  migration registers through `TESSARA_MODULE_CONTROL_ENDPOINTS` without new
  module-ID logic in Core.
- Acceptance check: pass when the constrained actor sees only allowed data and
  the migration plan needs no new shared Module Management implementation.
- Evidence:
  `artifacts/sprint-6c-final-closeout-2026-07-29-r2/playwright-retained-fresh.json`
  and `docs/architecture/independent-module-pathway.md`.

## 2026-07-27 - Sprint 6C Reusable Module Pathway Complete

- Status: closeout-ready. Sprint 6C now leaves a reusable migration pathway,
  not only two separately implemented modules.
- Core uses a definition-independent control-endpoint registry, schema-driven
  configuration rendering, generic configuration apply/read-back and security
  synchronization, and shared lifecycle/findings/diagnostics behavior.
  Dashboard and Scoped Records have no module-ID-specific Module Management
  path.
- The migration recipe is in
  `docs/architecture/independent-module-pathway.md`; the common/custom
  inventory is in
  `docs/audits/module-management-consistency-2026-07-27/README.md`.
- Implementation commit `b1b497689cec0fc0220b6ba26b53deed000a2978`
  established the pathway. Commit
  `f59468fc627d62fd2f8e5d629ba6b7714cc1bd4c` fixed hash synchronization
  for the shared module-detail tabs and is the exact deployed source; tree
  `963315ddc752f63cdf81be9d7f295be95e9b4cd1`.
- Final verification passed: full `scripts/validate.ps1`, final UAT and smoke,
  direct 61/61 Playwright, retained manifest-bound 61/61 Playwright,
  database isolation, and all ten degraded provider states. Durable proof is
  under `artifacts/sprint-6c-pathway-closeout/`.
- Next Sprint: Sprint 6D - Application Blueprint And Composition Automation
  Slice.

### Sprint Handoff / Demo Instructions

#### Shared Independent-Module Configuration

- Role: `admin`
- Paths:
  - `http://localhost:8080/administration/modules/tessara.dashboards#configuration`
  - `http://localhost:8080/administration/modules/tessara.reference.scoped-records#configuration`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open each path and compare Configuration, Application state, diagnostics,
     findings, dependencies, navigation, and Overview lifecycle interactions.
  3. Edit and save each module's declared configuration without changing its
     value.
  4. Disable and re-enable the product route; confirm the product navigation
     item follows enablement while Module Management remains available.
- Expected:
  - Panel structure, control behavior, statuses, error handling, and tab/hash
    navigation are identical.
  - Only manifest-owned labels, fields, values, dependencies, resources, and
    diagnostics differ.
- Acceptance check:
  - Pass when both modules complete the same workflow and no module-specific
    Core administration behavior is visible.
- Evidence location:
  - `artifacts/sprint-6c-pathway-closeout/playwright-final-fresh.summary.json`
  - `artifacts/sprint-6c-pathway-closeout/uat-final-fresh.json`
  - `docs/audits/module-management-consistency-2026-07-27/README.md`

#### Reusable Migration Path Review

- Role: implementer/reviewer; no application role is required.
- Paths:
  - `docs/architecture/independent-module-pathway.md`
  - `docs/audits/module-management-consistency-2026-07-27/README.md`
- Steps:
  1. Follow the ownership, manifest, configuration, endpoint, deployment,
     bootstrap, and test checklist.
  2. Compare each step with Dashboard and Scoped Records.
  3. Confirm the next module can register through
     `TESSARA_MODULE_CONTROL_ENDPOINTS` without a module-ID branch in Core.
- Expected:
  - The bounded shared work and the intentionally module-owned product work are
    explicit.
- Acceptance check:
  - Pass when a migration plan can be produced from the guide and requires no
    new Core Module Management screen or per-module control adapter.
- Evidence location:
  - `docs/architecture/independent-module-pathway.md`
  - `end2end/tests/modules.spec.ts`
  - `artifacts/sprint-6c-pathway-closeout/deployment-handoff-fresh.json`

## 2026-07-26 - Sprint 6C Implementation Complete

- Status: implementation and closeout-readiness verification are complete.
  Sprint 6C extracts Dashboards into an independently deployed full-stack
  module while preserving the established same-origin product URLs.
- Dashboard now owns its baseline migration, database, runtime/migration
  identities, API, composition reconciliation, native SSR pages,
  configuration, health, readiness, diagnostics, and deterministic seed.
  Core owns only routing, signed shell projection, action-bound authorization,
  Module Management, and the transition Components compatibility adapter.
- Dashboard placements retain typed Core-owned ComponentVersion references
  without Core-table joins or foreign keys. Authorized resolution covers the
  required provider, lifecycle, compatibility, identity, and owner-data
  states; unauthorized resolution remains nondisclosing.
- The approved degraded editor treatment is implemented: warning-tinted
  placement tile, one prominent warning icon in the tile, and a side sheet
  containing the full diagnostic, a large centered icon, and retry. Authorized
  provider outages preserve the saved placement title.
- Fresh Compose bootstrap is repeatable at receipt revision 1; Core,
  Dashboard, deployment-control, and Scoped Records images carry matching
  clean source labels; Dashboard database credentials fail against every
  non-Dashboard database.
- Validation completed: workspace formatting/clippy and Rust suites, the full
  PowerShell 7 validation wrapper (including release nondisclosure timing),
  fresh smoke and UAT, all 61 canonical Playwright tests with one worker and
  zero retries/skips/failures, desktop/mobile
  degraded-state browser review, retry, Dashboard-process outage containment,
  and post-restart data preservation. Durable proof is retained under
  `artifacts/sprint-6c-closeout/`; command-level details are in
  `docs/sprints/sprint-6c-verification.md`.
- The final source-exact deployment and retained machine evidence are bound to
  implementation commit `d56bc817332ce5fb8f75592bb8fa739fb303b215`
  and source tree `e7d590567b699fd276c703cea0c1be26b7e93b50`.
- Next Sprint: Sprint 6D - Application Blueprint And Composition Automation
  Slice.

### Sprint Handoff / Demo Instructions

#### Independent Dashboard Authoring And Viewing

- Role: `admin`
- Paths:
  - `http://localhost:8080/dashboards`
  - `http://localhost:8080/dashboards/26e397d0-e869-48cf-87a4-57816b167978`
  - `http://localhost:8080/dashboards/26e397d0-e869-48cf-87a4-57816b167978/edit`
  - `http://localhost:8080/dashboards/26e397d0-e869-48cf-87a4-57816b167978/view`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open **Dashboards**, then open **Demo Operations Dashboard**.
  3. Visit Detail, Editor, and Viewer; select a placement in the editor and
     use the placement controls.
  4. Save the unchanged layout and refresh the viewer.
- Expected:
  - All pages remain in the normal Tessara shell and use the existing
    same-origin URLs.
  - The Dashboard shows nine placements and the administrator retains the
    edit affordance.
- Acceptance check:
  - Pass when the same Dashboard identity and nine-placement composition
    survive refresh and the normal directory/detail/editor/viewer flow.
- Evidence location:
  - `artifacts/sprint-6c-closeout/smoke-fresh.json`
  - `artifacts/sprint-6c-closeout/uat-fresh.json`
  - Dashboard cases in `playwright-acceptance-fresh.json`

#### Module Management And Isolated Operations

- Role: `admin`
- Paths:
  - `http://localhost:8080/administration/modules`
  - `http://localhost:8080/administration/modules/tessara.dashboards`
- Steps:
  1. Open Module Management and select **Dashboards**.
  2. Inspect its independently deployed status, release, instance, database,
     configuration, health/readiness routes, diagnostics, capabilities,
     navigation, and Components dependency binding.
  3. Confirm the diagnostics identify Sprint 8A as the transition-binding
     migration target.
- Expected:
  - Dashboard appears once as an independently deployed module, replacing its
    transition inventory row without changing unrelated contributions.
- Acceptance check:
  - Pass when the module is healthy/enabled, its manifest and deployment
    projections agree, and no Dashboard product data is exposed in Core
    administration.
- Evidence location:
  - `artifacts/sprint-6c-closeout/deployment-fresh.json`
  - Module Management cases in `playwright-acceptance-fresh.json`
  - `scripts/verify-sprint-6c-isolation.ps1`

#### Scoped Access And Nondisclosure

- Role: constrained non-admin actor created by the permissions acceptance
  fixture; the scripted walkthrough is authoritative because the actor and
  disjoint Organization scopes are deliberately ephemeral.
- Paths:
  - `/api/dashboards`
  - `/api/dashboards/{dashboard_id}`
  - `/dashboards/{dashboard_id}/view`
- Steps:
  1. Assign Dashboard authority over Organization subtree A and Component
     authority over disjoint subtree B.
  2. Compare known and random out-of-scope ComponentVersion references.
  3. Open an authorized Dashboard containing a restricted placement.
- Expected:
  - The actor sees only authorized Dashboards. Both known and random
    unauthorized references collapse to the same nondisclosing result; the
    viewer retains only a redacted footprint and never executes hidden data.
- Acceptance check:
  - Pass when no identity, lifecycle, metadata, or rendered content leaks.
- Evidence location:
  - Permissions cases in
    `artifacts/sprint-6c-closeout/playwright-acceptance-fresh.json`
  - Module-contract and Core adapter tests recorded in
    `docs/sprints/sprint-6c-verification.md`

#### Components Provider Degradation

- Role: `admin`
- Paths:
  - `http://localhost:8080/dashboards/26e397d0-e869-48cf-87a4-57816b167978/edit`
- Steps:
  1. Recreate Core with
     `TESSARA_COMPONENTS_PROVIDER_STATE=unavailable`.
  2. Open the seeded Dashboard editor.
  3. Activate the warning icon on **Partner Profile**.
  4. Review the placement-issue side sheet and select **Retry resolution**.
  5. Restore `TESSARA_COMPONENTS_PROVIDER_STATE=available`.
- Expected:
  - Each affected tile retains its saved title, uses full-panel warning
    coloring, and shows one prominent warning icon with no inline diagnostic
    copy.
  - The side sheet contains the complete message, a large centered icon, and
    a semantic retry button.
- Acceptance check:
  - Pass when retry re-resolves the same nine saved footprints without
    changing the Dashboard or making Core unavailable.
- Evidence location:
  - `artifacts/sprint-6c-closeout/browser-outage-responsive-fresh.json`
  - Dashboard module/web resolution and degraded-editor tests

#### Dashboard Process Outage And Recovery

- Role: `admin`
- Paths:
  - `http://localhost:8080/dashboards`
  - `http://localhost:8080/administration/modules/tessara.dashboards#diagnostics`
  - `http://localhost:8080/reference/scoped-records`
- Steps:
  1. Run
     `docker compose -f deploy/sprint-6c/compose.yaml stop dashboards`.
  2. Open **Dashboards**, Module diagnostics, and Scoped Records.
  3. Run
     `docker compose -f deploy/sprint-6c/compose.yaml start dashboards`.
  4. Reopen the seeded Dashboard.
- Expected:
  - Dashboard routes show the authenticated Core-shell fallback; Module
    Management and Scoped Records remain usable.
  - Recovery restores the same Dashboard ID, nine placements, and manage
    affordance.
- Acceptance check:
  - Pass when there is no raw gateway failure, unrelated modules remain
    available, and Dashboard data survives the process restart.
- Evidence location:
  - `artifacts/sprint-6c-closeout/browser-outage-responsive-fresh.json`

#### Responsive Placement-Issue Sheet

- Role: `admin`
- Path:
  - seeded Dashboard editor above
- Steps:
  1. Degrade the Components provider and open a placement issue.
  2. Set the viewport to 390 by 844 pixels.
  3. Inspect the warning icon, diagnostic copy, and retry action.
- Expected:
  - The sheet fills the 390-pixel viewport without horizontal overflow; the
    96-by-96-pixel icon stays centered and retry remains visible.
- Acceptance check:
  - Pass when document/body scroll width remains 390 pixels.
- Evidence location:
  - `artifacts/sprint-6c-closeout/browser-outage-responsive-fresh.json`

### Acceptance Mapping

1. Separate Dashboard process, database, identities, manifest, configuration,
   health/readiness, API, and native UI:
   - Manual: **Independent Dashboard Authoring And Viewing** and **Module
     Management And Isolated Operations**.
   - Automated: deployment evidence, module inventory Playwright cases, and
     Dashboard module integration tests.
2. Dashboard credentials cannot read another database:
   - Manual: inspect the module database projection in **Module Management And
     Isolated Operations**.
   - Automated: `scripts/verify-sprint-6c-isolation.ps1`.
3. Placements use typed references without relational Core dependency:
   - Manual: inspect the Components binding and Dashboard diagnostics.
   - Automated: fresh-baseline/repository tests and the zero-Dashboard-table
     Core baseline check.
4. The Components adapter is versioned, action-bound, transition-only, and
   marked for Sprint 8A:
   - Manual: **Module Management And Isolated Operations**.
   - Automated: manifest, contract, and adapter tests.
5. Downstream grants bind actor, service, installation, audience, action,
   scope, revisions, expiry, and replay:
   - Manual: **Scoped Access And Nondisclosure**.
   - Automated: module-contract protocol tests and Core gateway tests.
6. Unauthorized resolution is nondisclosing and authorized resolution
   distinguishes every required state:
   - Manual: **Scoped Access And Nondisclosure** and **Components Provider
     Degradation**.
   - Automated: permissions Playwright cases plus the ten-state matrix in
     `artifacts/sprint-6c-closeout/degraded-states-fresh.json`.
7. Sprint 5A directory/create/detail/editor/viewer behavior is preserved:
   - Manual: **Independent Dashboard Authoring And Viewing**.
   - Automated: smoke, UAT, and all Dashboard Playwright cases.
8. Configuration/diagnostics remain in Core administration without Core
   product-data access:
   - Manual: **Module Management And Isolated Operations**.
   - Automated: module inventory tests and database-isolation script.
9. Dashboard and Components outages are contained:
   - Manual: **Components Provider Degradation** and **Dashboard Process
     Outage And Recovery**.
   - Automated: outage/module/web tests and retained browser evidence.
10. Fresh seed/bootstrap is deterministic and idempotent with expected
    one-row ledgers:
    - Manual: rerun `scripts/bootstrap-sprint-6c-deployment.ps1`.
    - Automated: retained deployment/smoke/UAT records and repeated bootstrap
      revision-1 proof.
11. Contract, permission, outage, isolation, API, SSR, responsive, smoke,
    UAT, and Playwright checks pass on exact clean source:
    - Manual: **Responsive Placement-Issue Sheet** and the complete handoff
      walkthrough.
    - Automated: `docs/sprints/sprint-6c-verification.md` and every artifact
      under `artifacts/sprint-6c-closeout/`, all bound to implementation and
      evidence commit `d56bc817332ce5fb8f75592bb8fa739fb303b215`.

## 2026-07-25 - Sprint 6C Kickoff

- Kickoff status: started the roadmap-selected Independently Deployed
  Dashboard Module Slice from clean `main` commit `c4e291c3`.
- Branch/worktree: `codex/sprint-6c` at
  `C:\Users\eric-dev\Projects\tessara-sprint-6c`.
- Plan: `docs/sprints/sprint-6c-plan.md`, including clause-level manual and
  automated acceptance mapping plus source-exact closeout readiness.
- Planned verification: formatting; module-contract, Dashboard, Dashboard web,
  API, and web tests; Sprint 6C Compose/bootstrap and database isolation;
  canonical Playwright; smoke; UAT; and retained source-provenance evidence.
- Immediate focus: inventory the existing Dashboard/Core/Components coupling,
  then define the typed transition ComponentVersion compatibility boundary
  before moving persistence or routes.

## 2026-07-25 - Sprint 6B2 Closeout

- Status: complete. Sprint 6B2 is closed against implementation/evidence commit
  `c21398e2e026b06411292db34fb6ac0e1a871dde`; the subsequent closeout-only
  commit records this handoff.
- Completed:
  - Secure administrator enrollment and reason-bearing recovery across the
    independent installation-control boundary.
  - Capability Floor v1, signed shell and authorization protocol contracts,
    scoped read/manage grants, revision invalidation, and replay protection.
  - Scoped Records configuration, directory/detail/create/edit,
    health/diagnostics, Organization scoping, and native Core-shell
    integration.
  - Approved Sprint 6B2 UI review, live conformance corrections, responsive
    containment, and dynamic module navigation labels.
  - Core and Scoped Records development migrations folded into their single
    fresh-install baselines; Installation Control already uses one baseline.
- Validation:
  - A source-exact Sprint 6B2 image was built from the closing commit and
    launched against empty Compose volumes. Core, installation control, and
    Scoped Records each applied exactly migration `001`.
  - Full API and web suites passed (144 API library tests plus all integration
    groups; 77 web tests). Module-contract, installation-control, and Scoped
    Records suites also passed.
  - Direct Playwright and the retained evidence runner each passed all 60
    tests with zero failures, skips, retries, or flakes. Fresh UAT and smoke
    evidence also passed.
  - Closeout evidence is retained under
    `artifacts/sprint-6b2-closeout/`; detailed commands, hashes, and ledger
    readback are in `docs/sprints/sprint-6b2-verification.md`.

### Sprint Handoff / Demo Instructions

#### 1. Guided administrator enrollment and recovery

Role: local installation operator, followed by the new administrator.

1. From the repository root, run
   `.\scripts\tessara.ps1 enrollment issue -Open`.
2. Copy the once-displayed claim secret into the prepared `/enrollment` page;
   verify the installation, claim, generation, and kind are already populated.
3. Enter a new email, display name, and password. Confirm the page states the
   password requirements, completes with **Enrollment successful**, and
   continues to `/login`.
4. When recovery is eligible, run
   `.\scripts\tessara.ps1 enrollment recover -Reason "Sprint 6B2 demo" -Operator "local-operator" -Open`
   and repeat the browser flow.

Expected result: the secret is shown only by the local command, never placed in
the URL or redisplayed by status; a successful enrollment creates the viable
Core Administrator, and recovery records its operator and reason.

Acceptance check: replay, expired/revoked/replaced claims, invalid handoffs, and
ineligible recovery all return the same non-disclosing designed failure path.
Evidence: `artifacts/sprint-6b2-closeout/e2e-fresh.json` and the enrollment
integration results in `docs/sprints/sprint-6b2-verification.md`.

#### 2. Capability floor and role administration

Role: Core Administrator. Path: `/administration/roles`.

1. Verify the Capability Floor v1 summary is covered and the designated role
   is **Core Administrator**.
2. Open that role and verify the floor-obligations note and authoritative
   `core:admin` capability.
3. Create or edit a scoped role. Confirm enabled module capabilities are
   assignable and that installation-global and Organization-scoped
   capabilities cannot be mixed except by the supported administrator rule.

Expected result: weakening the only viable designated enrollment role is
blocked; module capabilities remain independently assignable.

Acceptance check: role, capability, assignment, and designation changes advance
authorization revision and invalidate stale grants. Evidence: API role,
enrollment, and authorization integration groups plus the retained Playwright
run.

#### 3. Scoped Records configuration and application state

Role: Core Administrator. Path:
`/administration/modules/tessara.reference.scoped-records#configuration`.

1. Edit the display label and save configuration.
2. Confirm normalized validation remains valid and the navigation label
   updates without a restart.
3. Review the separate Application state panel, including configuration,
   health, navigation visibility, product-route enablement, and the health and
   diagnostics link.

Expected result: one module-owned schema validates the UI and machine-client
configuration; configuration does not silently change enablement.

Acceptance check: invalid configuration returns stable findings, while a valid
label persists and is projected into Core navigation. Evidence: API/web tests,
UAT, and Playwright.

#### 4. Scoped Records authorization and product workflow

Roles: Core Administrator and a scoped read/manage test user. Paths:
`/reference/scoped-records`, `/reference/scoped-records/records/new`, and a
record detail/edit route.

1. As the administrator, verify all seeded records are visible and create,
   edit, search, and Organization filtering work.
2. Assign disjoint read and manage roles to separate Organization subtrees.
3. As a read-only user, verify only records in the authorized subtree appear
   and create/edit controls and routes are unavailable.
4. As a manager, create or update a record in the managed subtree. Repeat the
   same idempotent request, then attempt to reuse its authorization for a
   changed payload.

Expected result: reads and mutations are constrained independently by current
Organization scope; the exact retry returns the recorded result and changed
replay fails.

Acceptance check: A/X versus B/Y isolation, current ownership, wrong audience
or action, stale revisions, and known-versus-random IDs fail closed. Evidence:
Scoped Records integration tests, UAT, smoke, and Playwright.

#### 5. Health and diagnostics

Role: authorized module user. Paths: `/reference/scoped-records/health` and
`/reference/scoped-records/diagnostics`.

1. Verify readiness, liveness, configuration, and Core authorization panels.
2. Refresh status, then inspect sanitized diagnostic context and download the
   sanitized diagnostics result.

Expected result: the module renders inside the Core-owned shell; diagnostics
contain stable status and revision values but no claims, signing material, Core
credentials, or browser cookies.

Acceptance check: direct module access without a valid `ShellContextV1` fails
closed, while the same-origin Core gateway succeeds. Evidence: module
integration tests and Playwright.

#### 6. Fresh Sprint 6B2 stack bootstrap

Role: developer or installation operator.

1. Run
   `docker compose -f deploy\sprint-6b2\compose.yaml down -v --remove-orphans`.
2. Build the stack with the source commit/tree provenance arguments documented
   in `docs/sprints/sprint-6b2-verification.md`, then run
   `docker compose -f deploy\sprint-6b2\compose.yaml up -d`.
3. Run `.\scripts\bootstrap-sprint-6b2-deployment.ps1`; rerun it to verify the
   existing receipt makes the second invocation a no-op.
4. Open `http://localhost:8080`.

Expected result: the public surface is Traefik/Core only; installation control
and Scoped Records remain private services with isolated database identities.

Acceptance check: all three migration ledgers contain only successful version
`1`, the deployment receipt is revision `1`, and the source/image evidence
matches the closing commit.

### Acceptance Mapping

- Installation-bound one-use claims and guided initial/recovery flows map to
  handoff section 1; automated proof is in API enrollment integration,
  installation-control state-machine/PostgreSQL tests, and retained
  Playwright.
- Capability Floor v1, viable designation, local identity, and signed fixture
  identity map to sections 1 and 2; automated proof is in API capability-floor
  and enrollment integration groups.
- Native module documents and the Core-owned shell boundary map to section 5;
  module integration and Playwright prove valid signed shell projection and
  direct-request failure.
- Short-lived installation/audience/action-bound grants, actor/service
  identity, declared contracts, scope, revisions, and replay protection map to
  section 4; module-contract, API authorization, and Scoped Records integration
  tests exercise each rejection class.
- Scoped Records configuration, enablement, directory, detail, create/edit,
  health, and diagnostics map to sections 3–5; web/API tests, UAT, smoke, and
  Playwright prove the application paths and designed states.
- A/X versus B/Y isolation, ownership, stale revision, wrong audience/action,
  and nondisclosure map to section 4; the database-backed Scoped Records
  integration suite is the authoritative process-boundary proof.
- Restore-safe enrollment state and stable module identity/database binding
  through upgrade/rollback map to sections 1 and 6; installation-control and
  Scoped Records PostgreSQL integration tests provide the automated proof.
- The fresh-baseline and source-exact closeout condition maps to section 6;
  deployment, smoke, UAT, and E2E evidence under
  `artifacts/sprint-6b2-closeout/` is bound to commit
  `c21398e2e026b06411292db34fb6ac0e1a871dde`.

- Next Sprint: Sprint 6C - Independently Deployed Dashboard Module Slice.

## 2026-07-23 - Sprint 6B2 Implementation Started

- Product-owner approval of the annotated UI delta records is recorded; production UI remains limited to those deltas.
- Implemented the first shared signed-protocol foundation in `tessara-module-contract`: purpose-bound Ed25519 envelopes, deterministic canonical signing bytes, `ShellContextV1`, authorization grants, independent capability/scope bindings, revision and audience/action validation, replay identifiers, and 60-second read/30-second mutation limits.
- Added verification-key-only development trust, valid signed shell/authorization/external-identity fixtures, a tampered negative fixture, and SHA-256 sidecars.
- Verification: `cargo test -p tessara-module-contract` passes 53 tests and doc-tests; `cargo check --workspace` passes.
- Next implementation focus: installation-control claim, eligibility, reservation, consumption, reconciliation, and recovery contracts plus the deployment-control migration/process.

## 2026-07-24 - Sprint 6B2 Secure Operation Slice Implemented

- Added signed shared protocol contracts, development trust fixtures, and
  native module shell validation/rendering helpers.
- Added the separate installation-control crate, deployment-database schema,
  operator CLI, private reservation/finalization service, one-way claim
  verification, and auditable lifecycle.
- Added Core Capability Floor v1, designated-role/viability enforcement,
  enrollment transactions, signed fixture-external binding, security
  revisions, declared authorization exchange, and same-origin module gateway.
- Expanded Scoped Records into an Organization-owned product slice with one
  configuration validator, signed-grant enforcement, atomic mutation replay,
  scoped APIs, and native operational/product routes.
- Applied the approved Roles, enrollment, module configuration, records,
  health, and diagnostics UI decisions without restoring rejected duplicate
  affordances or mixed Enrollment-column treatments.
- Added the private-network Sprint 6B2 Compose topology and retained database,
  contract, and regression evidence in
  `docs/sprints/sprint-6b2-verification.md`.

## 2026-07-23 - Sprint 6B2 Kickoff

- Status: kicked off Secure Module Operation Slice from clean `main` commit `7ed779d6`.
- Branch/worktree: `codex/sprint-6b2` at `C:\Users\eric-dev\Projects\tessara-sprint-6b2`.
- Plan: `docs/sprints/sprint-6b2-plan.md`.
- Planned verification: formatting; module-contract, installation-control, Scoped Records, API, and web tests; authorization/enrollment conformance; canonical Playwright; smoke; local launch; sprint UAT; and live restore/upgrade/rollback evidence.
- Immediate focus: freeze claim, capability-floor, `ShellContextV1`, authorization-grant, signing/trust, and module-configuration contracts, while producing the Sprint 6B2 HTML/CSS review suite required before visual implementation.

## 2026-07-23 - Sprint 6B1 Closeout

- Status: complete. Sprint 6B1 establishes the curated, single-host container deployment foundation and the first independently deployed reference module without turning Core into a container control plane.
- The deployment contract, `tessara-deploy` CLI, Compose/Traefik topology, isolated PostgreSQL databases and roles, scoped-records reference module, transactional receipt import, typed Module Management projections, and approved Screen A–C UI deltas are complete.
- The development schema was squashed before closeout: the final fresh deployment applies only `migrations/001_baseline.sql`. Superseded 002–004 files are removed and are not deployment inputs.
- The same parameterized module route now uses one shared page structure and styling for transition contributions and independently deployed modules, populated by a normalized typed view model. Serving state and deployment-change semantics come from authoritative projections rather than parallel view logic.
- Complete persisted `ModuleManifestV1` evidence supplies descriptor download and all detail tabs. Browser acceptance proves that each displayed source digest hashes the exact downloaded descriptor bytes.
- Verified third-party container admission, distribution, and lifecycle management are intentionally future development. Sprint 6B1 uses curated Tessara releases and does not require Cosign on the deployment host.

### Sprint Handoff / Demo Instructions

1. Open `http://localhost:8080` and sign in with the seeded local administrator account (`admin@tessara.local` / `tessara-dev-admin`).
2. Open **Module Management**. On **Modules**, verify the eight-row shared directory, independently deployed **Scoped Records**, authoritative serving badges, filters, copy controls, and the Runtime details side sheet.
3. Open **Scoped Records**. Compare its shared Definition and lifecycle panels with a transition contribution such as Forms; use the desktop tabs or mobile full-width selector, and verify explicit empty states where a module supplies no data.
4. Use **View source descriptor (JSON)** and **View deployment receipt**. Confirm the descriptor digest matches the source digest shown in the directory and detail view.
5. Open **Deployment**. Review resolved components, receipt history, applied change, provenance, health, local-time timestamps, and receipt download.
6. Visit `/reference/scoped-records/` through the same Tessara origin. The reference module, health probes, API, and retained record data are live; stopping the module produces the Core-rendered fallback while Module Management remains available.

### Acceptance Mapping

- **Curated contract and CLI:** deterministic validate/plan/apply/status/rollback behavior, stale or mismatched plans rejected, and sanitized receipts bound to installation/revision/idempotency evidence. Proven by module-contract/deploy tests plus `test-sprint-6b1-contract.ps1` and `test-sprint-6b1-live.ps1`.
- **Single-host runtime:** Core, Traefik, PostgreSQL, and scoped-records run under Compose with default-deny exposure and one Tessara origin. Proven by fresh deployment capture, Compose health, smoke, and live lifecycle checks.
- **Database isolation:** one PostgreSQL container hosts separate Core, deployment-control, and Module Instance databases with migration/runtime roles and negative cross-database proof. Proven by fresh baseline capture, API integration tests, UAT, and live lifecycle checks.
- **Module Management:** shared directory/detail composition presents persisted release, instance, manifest, configuration, diagnostics, navigation, and deployment receipt data without mutation authority. Proven by API/web tests and the complete canonical Playwright suite, including native no-JavaScript and responsive behavior.
- **Upgrade and rollback:** compatible upgrade and rollback retain Module Instance identity, database binding, route behavior, and stored record data. Proven by the live lifecycle script and deployment-ledger readback.
- **UI delta discipline:** only the approved Screen A–C additions and changes were implemented; existing route structure, components, icons, actions, and responsive patterns remain authoritative. Proven by the approved mockup records and browser acceptance.
- **Closeout evidence:** `artifacts/sprint-6b1/deployment-fresh.json`, `smoke-fresh.json`, `uat-fresh.json`, and `playwright-acceptance-fresh.json` are bound to the final fresh deployment and source commit.
- **Runner note:** bare root `npx playwright test` is not supported because `@playwright/test` is intentionally package-local. `scripts/validate-e2e.ps1` is the canonical manifest-bound runner and passes the complete suite.

- Next Sprint: Sprint 6B2 - Secure Module Operation Slice.

## 2026-07-22 - Sprint 6B1 Kickoff

- Status: kicked off Container Deployment Foundation from clean post-6A-UI `main` commit `a0cac408`.
- Branch/worktree: `codex/sprint-6b1` at `C:\Users\eric-dev\Projects\tessara-sprint-6b1`.
- Plan: `docs/sprints/sprint-6b1-plan.md`; roadmap now splits deployment foundation (6B1) from secure module operation (6B2).
- Planned verification: formatting; module-contract, deploy CLI, reference-module, API, and web tests; canonical Playwright; smoke; local launch; and sprint UAT.
- Immediate focus: freeze external-tool ownership and refine three runnable HTML/CSS Module Management review screens into bounded per-screen delta records before deployment implementation. The existing application remains the baseline; mockup omissions and approximations are not implementation instructions.

## 2026-07-22 - Sprint 6A-UI Fresh-Baseline Closeout

- Completed:
  - Closed the approved configuration-driven navigation composition, direct Core Admin routes, protected placement rules, responsive reader/manager workflows, Module Management directory/detail harmonization, role provenance, assignment-date readback, and the final directory/policy state treatments.
  - Preserved the Sprint 6A platform boundary: no Module Release/Instance persistence, materialization, Supervisor, gateway, OCI, module database, or runtime work entered this slice.
- Deferred follow-up (not Sprint 6A-UI scope):
  - Sweep established screens for visually similar controls that should consolidate into reusable, shared components; retain behavior while removing incidental styling drift.
  - Introduce the approved Sonner-based transient notification system for alert messages.
- Final fresh-baseline evidence was captured against implementation commit `4d1103fc91e93aacd201e03bfd8e479cd163faa0`:
  - Prior closeout databases and compose volumes were destroyed. The final local stack was launched with `scripts/local-launch.ps1 -FreshData`, applied only `001_baseline.sql`, seeded its authorized inventory, and exposed healthy API and Postgres services at `http://localhost:8080`.
  - `artifacts/sprint-6a-ui/deployment-fresh.json`, `smoke-fresh.json`, and `uat-fresh.json` passed with the required fresh data state.
  - `validate-e2e.ps1` passed all five reconciled UI scenarios with zero retries, skips, or failures, and published `end2end/sprint-6a-ui-ui-manifest.json` plus `artifacts/sprint-6a-ui/playwright-ui-fresh.json`.
  - `validate-resource-reference-nondisclosure.ps1` passed restricted known-versus-random reference timing checks and published `artifacts/sprint-6a-ui/resource-reference-nondisclosure-fresh.json`.
  - The final bounded source gate passed with disposable `TEST_DATABASE_URL`: evidence-script contracts, formatting, API/web/module-contract checks, 42 module-contract tests, 70 web tests, and 131 API library tests. Targeted fresh-start, SSR route, and historical-capacity proofs also passed.
  - The durable browser reconciliation replaces the obsolete literal reader/Any-of proof with protected Module Management placement, canonical lock treatment, reader access, and denied-write proof. It also proves both directions of the mixed-scope picker restriction, retains the `admin:all` exception, and verifies separate scoped/global role assignment.
- Fresh-sprint policy used for closeout:
  - This sprint starts from one squashed baseline migration and a freshly seeded database. Upgrade and rollback evidence are intentionally not produced; historical migration fixtures are not deployment inputs.
  - Historical dashboard capacity SQL remains only under a test fixture for direct preflight behavior; it is not a runnable migration or deployment-evidence input.
- Next Sprint: Sprint 6B - Module Runtime And Installation Infrastructure, after the fresh-baseline Sprint 6A-UI closeout evidence is certified.

### Sprint Handoff / Demo Instructions

### Navigation composition and protection

- Role: navigation manager (`modules:manage_navigation`).
- Paths:
  - `http://localhost:8080/administration/modules` → **Navigation**
- Steps:
  1. Inspect Main and Admin, then add a custom group, rename it, and move an optional destination into it.
  2. Reorder an optional destination, toggle its visibility, save, refresh, then move it back and delete the now-empty group.
  3. Attempt protected Main/Admin/Home and Admin-destination operations.
- Expected: custom changes persist atomically; protected operations remain unavailable or produce a clear non-mutating explanation; display changes never grant route access.
- Acceptance check: pass when a manager can complete optional-item/group work, but cannot delete required groups or violate protected placement/visibility rules.
- Evidence location: `scripts/uat-sprint.ps1`, `scripts/smoke.ps1`, `end2end/acceptance-manifest.json`, `end2end/sprint-6a-ui-ui-manifest.json`, and `artifacts/sprint-6a-ui/playwright-ui-fresh.json`.

### Reader access and direct Core Admin routes

- Role: module reader (`modules:read`) and a non-admin/no-capability comparison actor.
- Paths:
  - `http://localhost:8080/administration/modules`
  - `http://localhost:8080/administration/users`
  - `http://localhost:8080/administration/roles`
  - `http://localhost:8080/administration/node-types`
  - `http://localhost:8080/administration`
- Steps:
  1. Open Module Management as a reader and inspect Navigation without mutation controls.
  2. Direct-load each Core Admin route and request the removed `/administration` path.
  3. Repeat as the constrained non-admin actor.
- Expected: allowed routes retain SSR/direct-load ownership; `/administration` is an ordinary unmatched 404; display policy does not bypass authorization.
- Acceptance check: pass when reader/manager/no-capability gating and the removed landing route behave independently of the composed navigation display.
- Evidence location: smoke/UAT route assertions, permission scenarios, and the canonical Playwright run identified above.

### Module Management directory, details, and states

- Role: module reader; navigation manager for localized policy retry/save feedback.
- Paths:
  - `http://localhost:8080/administration/modules`
  - `http://localhost:8080/administration/modules/tessara.forms`
  - `http://localhost:8080/administration/modules/tessara.migration`
- Steps:
  1. Search by trimmed name and stable definition ID, combine status filters, trigger a no-match state, and clear filters.
  2. Inspect Forms, Responses, and retired Migration; use descriptor/digest reveal/copy controls.
  3. On Navigation, save an allowed change and observe saving/saved feedback; direct-load a nonexistent definition to inspect the not-found recovery action.
- Expected: canonical order is preserved, no-match differs from an empty inventory, retry/return actions are usable without JavaScript, and responsive cards remain operable at 1280/768/390 widths.
- Acceptance check: pass when search/status behavior adds no authority, state treatments remain distinct, and module detail/provenance information is readable and accessible.
- Evidence location: focused web tests, fresh smoke/UAT evidence, `end2end/sprint-6a-ui-ui-manifest.json`, `artifacts/sprint-6a-ui/playwright-ui-fresh.json`, and the final local stack at `http://localhost:8080`.

### Roles and assignments

- Role: admin.
- Paths:
  - `http://localhost:8080/administration/roles`
  - `http://localhost:8080/administration/users/<user-id>/edit`
- Steps:
  1. Inspect capability scope/provenance, digest reveal/copy controls, and assigned-user affordances.
  2. Edit a user’s roles, observe `Assigned on` for saved assignments and `Pending save` for a new selection, then save or cancel.
- Expected: ordinary mixed scope-aware/installation-global role bundles remain rejected; users may receive separate roles; existing assignment authorization and scope behavior are unchanged.
- Acceptance check: pass when scope/provenance and assignment timestamps are visible without granting additional capability or weakening the mixed-scope restriction.
- Evidence location: API/web tests, smoke/UAT role assertions, and the canonical Playwright run identified above.

### Acceptance Mapping

- Exit condition 1 — deterministic fresh baseline layout:
  - Manual demonstration: Navigation composition and protection, step 1.
  - Automated check: fresh deployment evidence plus `validate.ps1`.
- Exit condition 2 — stable required groups and atomic migration:
  - Manual demonstration: Navigation composition and protection, step 3.
  - Automated check: migration/deployment evidence and API tests.
- Exit condition 3 — required/custom group lifecycle:
  - Manual demonstration: Navigation composition and protection, steps 1–3.
  - Automated check: UAT and canonical Playwright group CRUD/protection identities.
- Exit condition 4 — Main, Home, and Organization rules:
  - Manual demonstration: Navigation composition and protection, step 3.
  - Automated check: smoke/UAT and UI manifest protection identities.
- Exit condition 5 — Admin destination protection:
  - Manual demonstration: Navigation composition and protection, step 3.
  - Automated check: smoke/UAT and canonical Playwright protection identities.
- Exit condition 6 — optional destination placement, order, and visibility:
  - Manual demonstration: Navigation composition and protection, steps 1–2.
  - Automated check: UAT and canonical Playwright placement identities.
- Exit condition 7 — persisted group labels/order:
  - Manual demonstration: Navigation composition and protection, steps 1–2.
  - Automated check: API tests and UAT refresh assertion.
- Exit condition 8 — composition remains separate from authorization:
  - Manual demonstration: Reader access and direct Core Admin routes, step 3.
  - Automated check: permission scenarios, smoke/UAT, and browser acceptance identities.
- Exit condition 9 — removed Administration landing route:
  - Manual demonstration: Reader access and direct Core Admin routes, step 2.
  - Automated check: smoke/UAT and canonical 404/direct-route assertions.
- Exit condition 10 — reader/manager policy access, conflict, and audit:
  - Manual demonstration: Navigation composition and protection, steps 1–3.
  - Automated check: API tests, UAT, and conflict/retry browser identities.
- Exit condition 11 — fail-closed responsive shell projection:
  - Manual demonstration: Reader access and direct Core Admin routes, steps 1–3.
  - Automated check: web tests, hydrate check, and UI manifest desktop/tablet/mobile identities.
- Exit condition 12 — accessible responsive Module Management hierarchy:
  - Manual demonstration: Module Management directory, details, and states, steps 1–3.
  - Automated check: web tests and the Sprint 6A-UI UI manifest.
- Exit condition 13 — deterministic directory search/status and no-JavaScript usefulness:
  - Manual demonstration: Module Management directory, details, and states, step 1.
  - Automated check: focused directory tests, smoke/UAT, and browser filter identities.
- Exit condition 14 — role assignment timestamp and pending state:
  - Manual demonstration: Roles and assignments, step 2.
  - Automated check: API/web tests and canonical Playwright role identities.
- Exit condition 15 — durable proof reconciliation:
  - Manual demonstration: review `docs/sprints/sprint-6a-ui-test-change-log.md`.
  - Automated check: `validate.ps1`, manifest validation, and `git diff --check`.
- Exit condition 16 — fresh closing deployment, source, SSR/hydration, and UI gates:
  - Manual demonstration: all handoff paths above on the running local stack.
  - Automated check: fresh evidence, smoke/UAT, canonical/UI Playwright, API/web tests, formatting, and source validation recorded below.

## 2026-07-16 - Sprint 6A-UI Implementation Complete (Historical Pre-Closeout Record)

- Implemented the approved Direction 1 Module Management directory controls as working trimmed, case-insensitive display-name/definition-ID search plus conjunctive status filtering with canonical order, responsive cards, a distinct clearable no-match state, and the canonical `Blocks` shell glyph.
- Added migration 004 and schema-v2 navigation management around stable required/custom groups and complete destination placements. The service enforces dense order, complete membership, protected placement/visibility, UUID-v4 custom identities, empty-only deletion, optimistic revision conflicts, and atomic audit-backed replacement while retaining legacy band rows only as inert rollback data.
- Replaced the actor shell's Administration landing item with capability-filtered direct User Management, Roles & Access, Node Types, and Module Management destinations. The shell now consumes arbitrary ordered groups, omits empty or unavailable groups/items, and retains Core-only fail-closed behavior.
- Implemented the responsive manager/reader composer with group carets, accessible lock-icon-only protection, exact capability-list eligibility in reader mode, cross-group/select and explicit ordering controls, mobile action sheets, dirty Save/Discard, revision reload recovery, and post-movement focus restoration.
- Added role capability Scope/Provenance columns and the approved non-redundant source labels with abbreviated reveal/copy digests. Exposed durable role-assignment creation time as `Assigned on`, preserving the earliest timestamp across assignment rewrites and showing pending/unassigned states without expanding assignment authority.
- Retained the current ordinary mixed-scope role restriction and recorded mixed-scope roles as future work requiring unambiguous per-capability or equivalent assignment semantics; a user can continue receiving separate scoped and installation-global roles.
- This pre-closeout proof was later superseded by the 2026-07-22 fresh-baseline closeout above: the migration history was squashed to `001_baseline.sql`, previous sprint databases were destroyed, and the final evidence ran against a newly seeded installation.

## 2026-07-16 - Sprint 6A-UI UX-Led Scope Clarification

- Clarified that Sprint 6A-UI is led by its UX outcome and is not limited to markup or CSS. Native interaction state and narrowly supporting behavior may change when they directly deliver an approved experience on the touched surfaces and receive proportional durable proof.
- Approved functional Module directory search by display name or stable definition ID plus availability/status filtering for All, Active in Core process, Unavailable, and Retired. Search and status combine without reordering the canonical inventory and include a distinct clearable no-match state.
- Replaced the blanket exclusion of functionality with the product boundary that unrelated feature expansion must be approved before inclusion. Installation/lifecycle operations, Release/Instance work, authorization changes, and unrelated workflow redesign remain unapproved.
- Kept tests as durable proof. The exact default seven-entry inventory, order, source data, authorization, and useful SSR/no-JavaScript output remain acceptance evidence; search/filter proof is additive and any affected accepted identity still requires a logged equal-or-stronger replacement.
- Completed the limited visual follow-up in Direction 1 assets 01, 10, and 16: desktop/mobile directory controls now show the approved search/status treatment, and the state reference distinguishes a clearable filtered no-match outcome from a genuinely empty inventory.
- Froze `core.admin.modules` / Module Management to Tessara's canonical `Blocks` icon and corrected only that sidebar glyph in Direction 1 assets 01, 03-08, and 13-15; unaffected mockups were not regenerated.

## 2026-07-15 - Approved Dynamic Navigation Model

- Replaced the planned band-preserving UI correction with an approved
  configuration-driven navigation model. `core.main` and `core.admin` are
  stable non-deletable Core group identities; custom groups can be created,
  renamed, reordered, and deleted when empty by effective global
  `modules:manage_navigation`.
- Approved free cross-group movement, visibility, and order management for
  optional destinations. Capability and module-availability filtering remain
  independent and authoritative, so display configuration never grants or
  revokes route/API access.
- Defined Home and Organization as Core Main placements. Home cannot be hidden
  or moved out; Organization cannot be removed or moved out but may be hidden.
  Operations becomes optional because Home is expected to subsume it.
- Defined User Management, Roles & Access, Node Types, and Module Management as
  Core Admin placements. They cannot be removed, hidden, or moved out of Admin
  but may be reordered. Required-group relabeling is supported by the model but
  is not an exit-critical requirement.
- Approved complete removal of the Administration navigation item,
  `AdministrationPage`, and exact `/administration` route with no redirect.
  Direct `/administration/users`, `/roles`, `/node-types`, and `/modules`
  routes remain with their current authorization and receive distinct active
  navigation identities.
- Confirmed that Core may hard-code the built-in destination catalog and
  protection rules while persisted installation policy owns group labels,
  group order, item group placement, visibility, and item order. The shell
  renders the policy projection rather than a hard-coded list.
- Audited the existing implementation: migration 003 constrains two groups and
  three bands; policy v1 stores contribution-only visibility/order; server and
  browser shell composition duplicate exact groups/ranks; and `/administration`
  is mounted independently. Sprint 6A-UI therefore now includes migration 004,
  versioned management/shell wires, composition, route, and durable-proof work
  in addition to targeted Module Management harmonization.
- Froze implementation defaults that do not require product choice: populated
  migration backfill versus fresh post-migration reconciliation; opaque custom
  group IDs and validated labels; management-only retention of empty groups;
  legacy band fields as immutable provenance rather than effective policy;
  revision-preserving conversion with a system audit; ordinary unmatched 404
  behavior for `/administration`; and separately versioned Sprint 6A-UI
  deployment/acceptance artifacts and repository-safe browser commands.
- Preserved closed Sprint 6A as historical evidence. Expected band and
  Administration assertions may change only through exact pre-approved
  test-log rows and equal-or-stronger migration, protection, group CRUD,
  cross-group, authorization, concurrency, SSR, and route-removal proof.
- One product decision remains before implementation and regenerated composer
  mockups: the deterministic initial placement of optional destinations,
  especially Datasets, under an initially Core-only Admin group.

## 2026-07-15 - Sprint 6A-UI Scope Correction And Targeted Audit

This entry remains authoritative for the captured Module Management content
defects and no-broad-redesign direction. Its band-preserving, unchanged-shell,
unchanged-route/API/persistence, and unchanged-60-test assumptions were
superseded by the approved dynamic-navigation entry above.

- Recorded the product owner's narrowed direction: Sprint 6A-UI corrects only
  presentation introduced by Sprint 6A on the Module Management directory and
  detail, navigation-policy controls, the Administration entry, and
  capability-provenance presentation. Existing Tessara pages are reference and
  regression surfaces only.
- Removed the earlier future-regrouping mockup idea, substantial-redesign
  direction, all-48-route visual baseline, shell work, and broad workflow work.
  Navigation groups, anchors, fixed Core items, existing bands, order and
  eligibility remain unchanged; no rebrand or shell/navigation redesign is
  authorized.
- Audited the live 1280×720 Module Management reader/manager surfaces and
  retained four current-run captures under
  `docs/audits/sprint-6a-ui-module-management-2026-07-15/`. The evidence shows
  severe inventory clipping, detail-content overlap and page-level horizontal
  scroll, dense policy rows, and weak runtime-versus-inventory hierarchy while
  confirming that useful semantic headings, regions, table relationships, and
  explicit state text already exist.
- Replaced the application-wide sprint plan and baseline with a bounded
  implementation contract and issue matrix. Existing Tessara framing, tokens,
  cards, information lists, tables, states, actions, and responsive rules are
  the implementation references; only narrowly scoped support may be added.
- Kept the 48-route inventory and exact 60-test manifest as parity context, not
  visual scope. Iteration uses targeted checks for the touched surface; the
  unchanged complete browser inventory plus fresh smoke/UAT and source gates
  run once after stabilization. A production or tracked-proof change after the
  canonical final pass invalidates the affected proof and requires the relevant
  targeted and commit-bound final gates again.
- Reaffirmed that tests are durable proof: no failing assertion, selector,
  timeout, fixture, identity, or screenshot may be changed merely to obtain a
  pass. Every existing-proof edit requires an approved log row and
  equal-or-stronger replacement proof; no application-wide visual baseline is
  introduced.
- Superseded checkpoint: the former statement that no blocker remained and one
  of three directory images could be selected as a full-page target is no
  longer current. Decision Gate 1 now freezes optional-destination placement;
  refreshed group-composer directions follow as the next review artifact.

## 2026-07-15 - Initial Broad Sprint 6A-UI Kickoff (Superseded Same Day)

This entry records the initial kickoff chronology only. Its broad visual scope
and Decision Gate 0 were superseded by the approved targeted contract above and
must not be used as implementation authority.

- Kicked off the sole roadmap sprint marked `(Next)` from clean post-closeout
  `main` commit `c37153b19787d4164eaccbb4752980772e6ec84a`, immediately after
  closed Sprint 6A commit `f145e059fc1f4d81c960cb35e586c802831ecea2`.
  The active branch is `codex/sprint-6a-ui` in
  `C:\Users\eric-dev\Projects\tessara-sprint-6a-ui`.
- Added the implementation contract at
  `docs/sprints/sprint-6a-ui-plan.md`, the source-backed audit ledger at
  `docs/sprints/sprint-6a-ui-baseline-inventory.md`, and the durable proof
  ledger at `docs/sprints/sprint-6a-ui-test-change-log.md`.
- Started implementation with a non-judgmental source inventory: the live
  native route tree mounts 48 route patterns plus the shared not-found
  fallback; the accepted schema-v2 browser manifest freezes 60 exact
  identities across seven files at SHA-256
  `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`.
- Preserved the closed Sprint 6A boundary. Sprint 6A-UI changes presentation
  and interaction only by default; routes, APIs, persistence, authorization,
  module contracts, navigation eligibility/bands/fixed Core items, lifecycle
  behavior, and Sprint 6B runtime scope remain unchanged.
- Established Decision Gate 0 before visual conclusions, mockups, or production
  UI code: confirm information-architecture exploration limits, visual
  direction, and the balance between all-route coherence and deep workflow
  redesign. Source inventory may continue without selecting those outcomes.
- Established durable test change control. Existing failures are presumed
  production regressions; tests may not be deleted, skipped, loosened,
  timeout/retry-dependent, or casually regenerated. Every existing-test edit,
  including a selector-only or visual-baseline change, requires an approved
  rationale and equal-or-stronger replacement proof in the sprint log.
- Planned proportional validation during implementation and one unchanged
  complete browser inventory against the final clean fresh release build,
  together with fresh smoke/UAT, source, SSR/hydration/console, accessibility,
  responsive, and narrowly reviewed visual proof. Sprint 6A populated-upgrade
  and rollback-package evidence is not rerun or overwritten unless approved
  scope expands into persistence, migration, API, authorization, or contracts.
- Planned source commands (not yet passed for Sprint 6A-UI) are
  `.\scripts\validate.ps1 -Fast`, `cargo fmt --all -- --check`, workspace
  check/Clippy with all features and locked dependencies, the WASM hydrate
  check, workspace tests excluding `tessara-api`, web tests, API library tests,
  web-crate boundary checks, `cargo audit --quiet`, and `git diff --check`.
  Bare root `npx playwright test` is unsupported/not run and claims no proof;
  the supported original-suite wrapper and a separate manifest-bound 6A-UI
  wrapper are the planned browser gates.
- Planned final deployment commands (also not yet passed) are locked end-to-end
  dependency/browser installation, `.\scripts\local-launch.ps1 -FreshData`,
  deployment capture at `http://127.0.0.1:8080`, `.\scripts\smoke.ps1`,
  `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8080"`, the unchanged
  `.\scripts\validate-e2e.ps1` 60-test pass, and the separate Sprint 6A-UI
  accessibility/viewport/visual wrapper. The literal kickoff-baseline
  `localhost` UAT form is diagnostic only because retained deployment evidence
  binds the exact host used at capture.
- Kickoff verification completed: `main` and `origin/main` matched the clean
  sequencing commit before branch creation; exactly one roadmap heading was
  marked `(Next)`; the new branch/worktree were clean; the 48 route patterns
  and 60/7 manifest identity/hash were derived from repository sources. No
  runtime validation is claimed for this documentation-only kickoff; planned
  commands remain explicitly unpassed until their proportional or final gate.
- Immediate focus after the three product decisions: reinforce the plan with
  the approved visual brief and exact evidence protocol, capture representative
  before states, freeze the prioritized issue matrix, and create UI/UX examples
  before the first production UI slice.

## 2026-07-15 - Sprint 6A Final Closeout

- Completed the Module Contract and Core Control Plane slice. User-facing
  changes are additive administration features: the Core-owned Module
  Management surface, its navigation-policy controls, and capability-provenance
  display in role management. All pre-existing product routes, their default
  relative navigation order before an authorized policy change, authorization
  behavior, and supported workflows remain unchanged.
- Established one durable Application Installation and Core runtime
  observation; exact versioned Manifest, Feature Declaration, contract,
  semantic-destination, typed-reference, and real `ModuleRelease`/
  `ModuleInstance` public types; and a seven-entry transition catalog. Sprint
  6A creates no Release/Instance table, row, mutation, provider, Supervisor
  process, module database, installation, materialization, or runtime claim.
- Forms, Workflows, Responses, Datasets, Components, and Dashboards are
  explicitly `transitional_in_process` and not independently deployable.
  Migration is `retired` historical/support inventory and has no current
  feature, contract, capability, route, navigation contribution, provider,
  configuration, destination, or action.
- Added native SSR Module Management directory/detail pages and exact
  human/machine descriptor parity. The fixed Module Management item is in the
  `Admin` group after Datasets. Effective installation-global `modules:read`
  exposes the item and read surfaces; `modules:manage_navigation` implies read
  and alone enables policy mutation; the separate Administration item remains
  `admin:all`-only. Navigation contributions can move only inside their
  existing Core-assigned bands; Core items, groups, anchors, and cross-band
  placement remain immutable.
- Core remains authoritative for roles and assignments. Ordinary roles cannot
  mix scope-aware and installation-global capabilities; `admin:all` is the sole
  universal-sentinel exception and the complete bundle remains
  installation-global. Module descriptors and navigation policy cannot mutate
  roles or product authorization.
- Deterministic built-in seed data is versioned package data and may be updated.
  The Sprint 6A package intentionally converges only the exact membership of
  built-in roles named `admin`, `operator`, and `respondent`; it preserves role
  rows/IDs, assignments, accounts, every user-managed credential, user-role
  links, every user-managed role/mapping, and every session column except the
  activity-driven `auth_sessions.last_seen_at`. The separate declared startup
  normalization may replace only the development admin credential. Those two
  narrow exceptions and the built-in membership replacement are executable
  upgrade proof, not a promise that all seed data remains historically frozen.
- Commit `6580b040236f563c30b5162fa833d7b0fed16478` is the reviewed production,
  canonical-fixture, and exact 60-scenario browser-inventory acceptance
  boundary. Evidence-only hardening then retained legitimate empty process
  logs (`fa197be6`), exercised a valid historical Forms write (`f832cecb`),
  froze flat response and one-element request-array wires (`86e22c3f`),
  normalized Docker 29 optional runtime defaults (`5bb81382`), and preserved
  canonical timestamp strings during live revalidation (`9ba79752`). None of
  those later commits changed production code, accepted fixture bytes, or a
  browser identity. Canonical Gate 3–6 evidence binds the clean commit
  containing this closeout entry; all earlier commit-suffixed artifacts are
  diagnostic only.
- Reconciled 40 test-change rationale rows: 32 dated 2026-07-14 and 8 dated
  2026-07-15. The accepted browser inventory remains 60 exact identities across
  seven files. No test was deleted, skipped, renamed, made retry-dependent,
  filtered, loosened, or given a longer timeout to obtain a pass; the temporary
  Dataset timeout was removed and its exact scenario passed under the unchanged
  30-second default.

### Validation And Retained Proof

- Gate 1 source/contract proof passed: formatting, all-feature check, strict
  all-target Clippy, 42 module-contract tests, native/WASM web-crate boundary
  checks, local-launch/deployment/Playwright/nondisclosure/rollback/acceptance
  self-tests, dependency audit, and diff hygiene. `cargo audit` found zero
  vulnerabilities; four explicitly allowed transitive warnings remain
  documented by the repository policy.
- Gate 2 integration proof passed: `scripts/validate.ps1` completed in about
  29m46s, and `cargo test --workspace --all-features --locked` completed in
  about 26m57s. The exercised graph includes 61 web tests and 192 API tests.
  The release-only resource-reference timing test executed samples rather than
  returning a debug pseudo-pass.
- Gate 3 passed on three separate databases: the populated Sprint 5A upgrade
  test passed 3/3 with zero ignored or filtered cases; the compatibility package
  passed `PackageOnly` and `CompatibilityOnUpgraded`; a custom-format backup of
  the independent Sprint 5A demo source restored to an exact fingerprint; and
  the original Sprint 5A executable passed `OriginalAfterRestore`. Canonical
  evidence is `compatibility-rollback/manifest.json`,
  `rollback-package-only.json`, `rollback-compatibility-upgraded.json`,
  `pre-upgrade-backup.dump`, `rollback-restore-evidence.json`, and
  `rollback-original-restored.json` under `artifacts/sprint-6a/`.
- Gate 4 passed against the restored demo target after the clean closing image
  applied migration 3 with `-SkipSeed`. Deployment capture, smoke, UAT, the
  exact 60/60 Playwright inventory, and 200-known/200-random nondisclosure
  sampling per restricted state passed. Browser evidence reports zero skipped,
  unexpected, flaky, filtered, or retried tests. Canonical files use the
  `deployment-upgraded`, `smoke-upgraded`, `uat-upgraded`,
  `playwright-acceptance-upgraded`, and
  `resource-reference-nondisclosure-upgraded` names documented in the
  deployment-evidence contract.
- Gate 5 independently passed after `local-launch.ps1 -FreshData` created a new
  Compose database. The equivalent fresh deployment, smoke, UAT, 60/60 browser,
  and nondisclosure sets passed with the same zero-skip/retry requirements and
  database-derived `fresh` classification.
- Gate 6 reran Gate 1 against the final tracked closeout commit, including every
  evidence self-test and `cargo audit --quiet`; `git diff --check` passed and
  final `git status --short` emitted no lines. The final fresh application is
  left healthy at `http://localhost:8080/` for review.
- Seven real-runtime conformance areas remain explicitly deferred—not passed—to
  Sprint 6B: Supervisor Manifest execution, real provider/consumer binding,
  gateway routing, cross-process grants/audience/replay/freshness, per-instance
  database isolation, real outage/health fallback, and the OCI platform
  conformance suite. Their accountable roles and executable Sprint 6A
  substitutes remain in the plan.

### Sprint Handoff / Demo Instructions

The retained final deployment is the read-only review target. Any walkthrough
that creates roles/accounts or changes navigation policy must use a disposable
manual-test deployment and the exact setup/cleanup procedure in the Sprint 6A
plan's `Disposable Reader/Manager And Provenance Fixtures` section. Never add
manual actors to, or mutate policy in, either canonical upgraded/fresh evidence
database.

#### Module Inventory And Truthful Transition State

- Role: seeded admin, then a disposable installation-global
  `modules:read`-only actor.
- Paths: `/administration/modules`,
  `/administration/modules/{definition_id}`, `GET /api/admin/modules`, and the
  matching detail/descriptor APIs.
- Steps:
  1. Sign in as `admin@tessara.local` and open Module Management from the
     `Admin` group.
  2. Inspect Forms, Workflows, Responses, Datasets, Components, and Dashboards;
     compare their directory/detail fields with the inventory and exact
     descriptor responses.
  3. Verify each is labeled transitional/in-process and makes no Release,
     Instance, install, enablement, readiness, or health claim.
  4. Open Migration and verify `Retired`, the withdrawal narrative, and the
     absence of route, navigation, provider, feature, contract, capability,
     configuration, destination, and action declarations.
- Expected result: human and machine projections agree, six current entries
  remain honest in-process contributions, and Migration remains inert history.
- Acceptance check: pass only when all seven entries and their source digests
  match and no UI/API field implies a deployable runtime.
- Evidence: `deployment-upgraded.json`, `deployment-fresh.json`,
  `playwright-acceptance-{upgraded,fresh}.summary.json`, contract fixtures, and
  module inventory/API tests.

#### Global Read Versus Navigation Management

- Roles: disposable reader with only `modules:read`; disposable manager storing
  only `modules:manage_navigation`; seeded operator as the product-only
  comparison. Scoped-only module authority and no-access cases remain automated
  proof because the supported role API correctly rejects a scoped assignment
  for an installation-global module capability.
- Paths: `/administration/modules`, `GET /api/admin/navigation-policy`,
  `PUT /api/admin/navigation-policy`, and `GET /api/shell/navigation`.
- Steps:
  1. Create the prefixed roles/accounts through the documented admin APIs with
     no scope node; confirm the manager role has no separately stored read row.
  2. Sign in as reader. Verify the `Admin` group and fixed Module Management
     item appear while Administration is absent; inspect inventory, descriptor,
     and policy readback; confirm no enabled mutation control and a direct
     policy `PUT` returns `modules_manage_navigation_global_required`.
  3. Sign in as manager. Verify read surfaces appear through implication and
     show/hide/reorder controls plus the policy `PUT` are enabled.
  4. Sign in as the seeded operator and verify that product authority alone does
     not expose or authorize Module Management. Inspect the retained automated
     result for scoped-only injection and no-access actors rather than bypassing
     the supported API in the manual environment.
- Expected result: read controls discovery only; manage implies read; neither
  capability grants Administration or product authority.
- Acceptance check: pass only if UI visibility, direct routes, API reads, and
  API writes independently match the named actor matrix.
- Evidence: both Playwright summaries, `modules.spec.ts`, module API permission
  tests, and `docs/playwright-permissions-scenarios.md`.

#### Band-Restricted Navigation Without Authorization Changes

- Role: disposable global manager; repeat direct-load with an actor already
  authorized for Forms.
- Paths: Module Management navigation policy, `/forms`, desktop shell, and
  mobile shell.
- Steps:
  1. Save the original policy/revision; hide Forms and move Dashboards before
     Components within its current band.
  2. Reload desktop and mobile shells and require the same persisted visibility
     and relative order.
  3. Direct-load `/forms` as an authorized actor and require the route/API to
     remain usable even though its contribution is hidden.
  4. Attempt to move Dashboards before Operations, Forms after Operations, or
     Datasets after Module Management; require atomic
     `navigation_policy_band_change_forbidden` rejection and unchanged state.
  5. Attempt to mutate `module_management`; require atomic
     `navigation_policy_core_item_immutable` rejection. Exercise a stale
     revision and require `409 navigation_policy_revision_conflict`.
  6. Restore the original policy and verify persistence before cleanup.
- Expected result: policy changes presentation only inside existing bands;
  groups, Core anchors, authorization, and audit transactionality remain intact.
- Acceptance check: pass only when positive changes persist, every forbidden
  request leaves the full prior projection unchanged, and authorization is
  identical before/after.
- Evidence: both Playwright summaries, policy API integration/audit tests, and
  deployment-bound shell/navigation snapshots.

#### Capability Provenance And Scope Rules

- Role: seeded admin and disposable `manual-sprint-6a-provenance` role.
- Path: `/administration/roles` and the role/capability admin APIs.
- Steps:
  1. Inspect `forms:read`; require its Core/Forms transition provenance,
     installation/scope metadata, and current provider state.
  2. Add/remove a compatible scope-aware capability on the disposable role and
     save the intended bundle; require descriptors and navigation policy to
     remain byte-for-byte unchanged.
  3. Attempt an ordinary scope-aware plus installation-global mixed bundle and
     require atomic rejection with the prior role unchanged.
  4. Inspect the seeded `admin` role, which contains only `admin:all`, as the
     sole universal-sentinel exception; require the complete role to classify
     installation-global.
  5. Clean up only the prefixed manual fixtures through the evidence-bound
     database triple and require zero remaining rows.
- Expected result: Core owns role mutation, provenance is discoverable, ordinary
  mixed scope modes fail, and only `admin:all` receives the approved exception.
- Acceptance check: pass only when failed changes are atomic and no role action
  changes a module descriptor or navigation policy.
- Evidence: role API integration tests, both permissions Playwright summaries,
  and the versioned built-in seed digest in both deployment records.

#### Semantic Destinations And Typed References

- Role: seeded admin plus the existing respondent/delegation fixtures.
- Paths: `POST /api/platform/destinations/resolve`,
  `POST /api/platform/resource-references`, and
  `POST /api/platform/resource-references/resolve`.
- Steps:
  1. Read the exact Application Installation ID from module inventory and
     resolve schema-v1 `core_installation` route `forms.detail` with a typed
     `form_id`; require only the existing same-origin `/forms/{form_id}` path.
  2. Construct and resolve the checked `tessara.transition.form` reference for
     `Platform reference fixture` / `platform-reference-fixture`.
  3. Resolve the deterministic owned and delegated
     `tessara.transition.response` references, then repeat with random UUIDs and
     mismatched owner/type/installation/schema/fields.
  4. Require every reference to remain owned by the exact Core Application
     Installation, never a fictional Module Instance; compare restricted known
     and random responses byte-for-byte.
- Expected result: destinations are semantic and same-origin; typed references
  cannot be reinterpreted; authorization precedes existence lookup and returns
  independent seven-dimension resolution outcomes.
- Acceptance check: pass only when all positive identities resolve exactly and
  restricted known/random envelopes and timing remain non-disclosing.
- Evidence: platform/reference API tests and
  `resource-reference-nondisclosure-{upgraded,fresh}.json` plus sidecars.

#### Native SSR And Existing Product Regression

- Roles: admin, operator, respondent, and the established scoped/delegated
  fixtures.
- Paths: Home, Organization, Forms, Workflows, Responses, Operations,
  Administration, Datasets, Components, Dashboards, and Module Management.
- Steps:
  1. Disable JavaScript and direct-load/refresh every native route family;
     compare existing hydrate-dependent routes with their frozen characterized
     state rather than imposing new data-complete SSR.
  2. Re-enable JavaScript and repeat representative list/detail/create/edit/
     execute flows for each authorized actor.
  3. Inspect desktop/mobile navigation, route responses, browser console, and
     network requests.
- Expected result: native document ownership, hydration parity, existing
  authorization and relative navigation, responsive layout, and all supported
  workflows remain intact; no `/bridge/*` request or unexpected console error
  occurs.
- Acceptance check: pass only when the exact 60-test manifest runs unfiltered
  with one worker, zero retries/skips/flakes, and both smoke/UAT sets pass.
- Evidence: upgraded and fresh smoke/UAT JSON plus sidecars and all four
  Playwright artifacts for each state.

#### Upgrade, Rollback, And Fresh-State Handoff

- Role: release reviewer/database operator; final UI walkthrough uses seeded
  admin on the fresh deployment.
- Paths: `artifacts/sprint-6a/`, `http://localhost:8080/`, and `/health`.
- Steps:
  1. Verify rollback package manifest/payload digests bind the final commit and
     exact Sprint 5A source; inspect `PackageOnly` and
     `CompatibilityOnUpgraded` results.
  2. Verify the custom backup and structured restore record bind the migration-2
     source and restored target with identical all-table fingerprints; inspect
     `OriginalAfterRestore`.
  3. Compare upgraded deployment/acceptance records: they must bind the external
     restored database, classify `upgraded`, contain pre-migration product rows,
     and show no demo seed invocation.
  4. Compare fresh records: they must bind the new Compose database directly,
     classify `fresh`, and contain no pre-migration product rows.
  5. Confirm final `/health` returns `200`/`ok`, `/` returns `200`, and leave the
     fresh application running for reviewer use.
- Expected result: historical code works on the compatible upgraded ledger,
  backup restore recovers the original ledger, current code preserves upgraded
  product assets, and a clean install independently works.
- Acceptance check: pass only when every artifact digest and commit/tree/image/
  container/user/database binding revalidates and the tracked tree is clean.
- Evidence: the complete canonical Gate 3–5 artifact index in
  `docs/sprints/sprint-6a-deployment-evidence.md` and the Gate 6 command record.

### Acceptance Mapping

| Roadmap exit-condition clause | Manual handoff section | Durable automated proof |
| --- | --- | --- |
| Fixed Module Management is discoverable by effective global read | Module Inventory; Global Read Versus Navigation Management | `modules.spec.ts` global-read scenario, module inventory/API tests, both deployment records |
| Six current features are in-process contributions rather than Module Instances | Module Inventory And Truthful Transition State | exact transition fixtures/contract tests, directory/detail parity scenario, both catalog snapshots |
| Features, contracts, capabilities, findings, and policy are readable without mutation controls | Module Inventory; Global Read Versus Navigation Management | descriptor parity/API tests and read-only actor browser/API negative-write proof |
| Global manage changes visibility/order without granting authorization | Band-Restricted Navigation Without Authorization Changes | keyboard policy scenario, navigation API/audit tests, shell/permissions coverage |
| Migration is retired and inert | Module Inventory And Truthful Transition State | retired fixture/catalog tests, directory/detail parity, both deployment snapshots |
| Provenance and scope rules remain Core-owned and fail closed | Capability Provenance And Scope Rules | role/capability API tests, permissions browser scenarios, exact seed contracts |
| Semantic destinations and typed references do not leak existence or deployment URLs | Semantic Destinations And Typed References | platform/reference API tests and both release nondisclosure artifacts |
| Existing product routes and workflows continue to work | Native SSR And Existing Product Regression | exact 60-test/seven-file upgraded and fresh Playwright sets plus both smoke/UAT sets |
| Upgrade, compatibility rollback, restore, and fresh installation are proven separately | Upgrade, Rollback, And Fresh-State Handoff | Gate 3 package/restore evidence plus distinct Gate 4 upgraded and Gate 5 fresh artifact sets |

- Next Sprint: Sprint 6B: Module Runtime And Installation Infrastructure Slice.

## 2026-07-14 - Sprint 6A Implementation-Contract Hardening

- Hardened the active sprint plan into an implementation contract with exact endpoint/authority behavior, source/digest rules, transition catalog, frozen regression matrix, acceptance-to-proof mapping, upgrade/rollback procedure, audit behavior, risk/abort conditions, conformance deferrals, and ordered validation gates.
- Recorded tests as durable proof: frozen failures default to production regressions; existing expectations cannot be deleted, skipped, loosened, retried, or regenerated to obtain green; accepted fixtures are immutable; every pre-acceptance correction and later expectation change is reviewable in `docs/sprints/sprint-6a-test-change-log.md`.
- Froze seven canonical transition sources and exact SHA-256 sidecars. Forms, Workflows, Responses, Datasets, Components, and Dashboards have non-empty discovery narratives and exact catalog declarations. Migration is the sole `retired` source and has no live feature, contract, dependency, resource, route, navigation, capability, configuration, provider, or execution declaration.
- Kept descriptor-source proof separate from later normalized catalog projection proof: `transition_internal_only`, Core-installation resource ownership, and `transition_destination_retired` are not fabricated as source JSON fields.
- Preserved existing Core capability keys/descriptions and user-managed role mappings. Product confirmed that deterministic built-in seed membership may update. The historical fixture's truthful Sprint 5A 20-capability/admin-20/operator-10/respondent-2 state is independently frozen as `sprint-5a-role-capabilities-v1+sha256.7725e889996a` / `7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988`; the reviewed current contract is `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68` / `2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600`: admin stores only `admin:all`, operator stores its exact established 10, and respondent stores its exact established 2. Only membership for those names is replaceable; role rows/IDs, assignments, accounts, sessions, user-role associations, and all user-managed roles/mappings remain exact invariants. The upgrade proof covers transactional replacement, permitted running-installation drift reconverging at startup, restart/concurrency convergence, and a fresh exact set. The destructive proof now fails instead of skipping when its dedicated URL is absent and rejects an unsafe database name before reset. The plan also defines installation-global module capability scope metadata, manage-implies-read behavior, ordinary mixed scope-mode rejection, the confirmed sole `admin:all` mixed-bundle exception (always installation-global), and fail-closed scoped-role tests.
- Documented two red/explicit baseline facts instead of weakening proof: the Dataset preview and revision-edit Leptos routes need matching Axum document-route registrations before the navigation refactor, and existing hydrate-dependent screens retain their characterized SSR/no-JavaScript behavior rather than acquiring unplanned data-complete SSR.
- Corrected rollback evidence: an unmodified Sprint 5A package cannot recognize applied SQLx migration 3. Sprint 6A must retain/test a Sprint 5A-code compatibility package carrying immutable migrations 1–3; the original package is used only after pre-upgrade backup restore.
- Focused validation completed:
  - `cargo fmt --all -- --check` - passed.
  - `cargo test -p tessara-module-contract --locked` - passed: 26 unit and 4 integration tests; 0 failed, ignored, measured, or filtered.
  - `.\scripts\check-web-crate-boundaries.ps1` - passed for Windows and WASM targets.
  - PowerShell parser checks for `scripts/validate.ps1` and `scripts/check-web-crate-boundaries.ps1` - passed.
  - all relative links in changed documentation - resolved.
  - `git diff --check` - passed; only expected Git line-ending notices for the CRLF-governed PowerShell files were emitted.
- `.\scripts\validate.ps1 -Fast` was attempted with a 15-minute bound and did not complete before the bound; it emitted no assertion failure, left no Cargo/rustc/API process, and is not counted as passed evidence. `TEST_DATABASE_URL` was not set, so full database, populated-upgrade, rollback/restore, upgraded Gate 4, fresh Gate 5, smoke, UAT, Playwright, and nondisclosure gates were not attempted or claimed.
- Product confirmed that contribution reordering is existing-band-only. The plan now freezes Core-owned bands on either side of Operations and between Administration and Module Management, rejects group/band/cross-anchor mutation atomically, and requires durable positive and negative policy proof.
- Product confirmed a permanent Core Module Management item in the `Admin` group, appended after Datasets to preserve every existing item's relative order. Effective global `modules:read` makes the item and read surfaces visible; `modules:manage_navigation` enables mutation controls and implies read; `admin:all` implies both. The separate Administration item remains `admin:all`-only. Permission proof must cover item/group visibility, read-only presentation, enabled controls, direct writes, Core-item immutability, and desktop/mobile parity for every named actor fixture.
- All Sprint 6A navigation product decisions required by the implementation contract are resolved.
- Replaced deployed-gate self-attestation with retained schema-v1 evidence. Release images carry the clean source commit/tree, dirty-state, and release-profile labels; capture verifies the running immutable image, live BaseUrl, API-to-`current_database()` Application Installation binding, successful migrations exactly 1–3 against current SQL SHA-384 checksums, the computed versioned built-in seed digest, exact transition-only catalog, and database-derived upgraded/fresh history. Smoke, UAT, Playwright acceptance, and non-disclosure timing now reject missing, stale, opposite-state, or SHA-mismatched evidence before exercising the deployment. Targeted local diagnostics require explicit `-DevelopmentMode` and are not acceptance evidence.
- Corrected the closeout database sequence before commit-bound proof: the destructive `SPRINT_6A_UPGRADE_DATABASE_URL` fixture is retained only for representative invariant and `CompatibilityOnUpgraded` checks. Gate 3 backs up a separate Sprint 5A demo source, restores it into a disposable target, and validates that target with `OriginalAfterRestore`; Gate 4 then lets the clean closing image apply migration 3 under `-SkipSeed`. Deployment classification plus smoke/UAT/Playwright prove the pre-migration actors and demo assets survived, and upgraded acceptance never calls `/api/demo/seed`; any Gate 4 demo mutation disqualifies the pass. The validated rollback package's exact historical binary plus `original-migrations` and `seed-demo` is the reproducible recovery path if the migration-2 source is lost.

## 2026-07-13 - Sprint 6A Module Contract And Core Control Plane Kickoff

- Sprint: `Sprint 6A: Module Contract And Core Control Plane Slice`, selected from the sole roadmap heading marked `(Next)`.
- Kickoff status: started from clean `main` at `3625d4de52c5856e4ac3bc642a9422a029e9f375`; branch/worktree setup and roadmap review are complete, and implementation has begun with the pure module/transition contract boundary.
- Branch: `codex/sprint-6a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6a`
- Plan file: `docs/sprints/sprint-6a-plan.md`
- Agreed product decisions:
  - Sprint 6A defines real `ModuleRelease` and `ModuleInstance` contract types; persistence and mutation for those records begin in Sprint 6B.
  - transitional Migration is `retired`: the former surface was deliberately withdrawn and remains discoverable only for historical/support context, with no current route, provider, navigation item, or executable destination; restoration requires a new product decision.
  - navigation ordering is configurable only within existing Core-assigned bands; Forms/Workflows/Responses cannot cross Operations, Components/Dashboards remain after Operations, Datasets remains between Administration and Module Management, Core items remain fixed, and grouping changes are deferred.
  - `modules:read` and `modules:manage_navigation` are global capabilities, and `modules:manage_navigation` implies `modules:read`.
  - Module Management is a permanent Core item in the `Admin` group after Datasets; effective global `modules:read` shows the item/read surfaces, while `modules:manage_navigation` gates policy mutation and `admin:all` implies both.
- Planned check-only Rust and repository gate (not yet claimed as passed):
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --all-features --locked`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - `cargo test -p tessara-module-contract --locked`
  - set `TEST_DATABASE_URL` to a disposable test database, then run `.\scripts\validate.ps1`
  - `cargo test --workspace --all-features --locked`
  - `.\scripts\check-web-crate-boundaries.ps1`
  - `cargo audit --quiet`
  - `git diff --check`
- Planned stateful and deployed validation (not yet claimed as passed): upgrade a populated Sprint 5A database without reset; verify preserved product data, capability mappings, sessions, audit identities, and every pre-existing navigation item, with only the approved fixed capability-filtered Module Management addition; exercise restart plus repeated/concurrent catalog synchronization; then run `.\scripts\local-launch.ps1 -FreshData`, capture and validate the machine-derived fresh deployment record, and run smoke, UAT, and `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath 'artifacts/sprint-6a/deployment-fresh.json' -ExpectedDataState fresh -EvidencePath 'artifacts/sprint-6a/playwright-acceptance-fresh.json'` against that exact deployment. Retained deployment and Playwright artifacts refuse implicit replacement; failures cannot erase prior green proof. This kickoff-era fresh-only sketch is superseded by the active plan's ordered Gates 1–6, including rollback/restore and the separate upgraded Gate 4 before Gate 5.
- Evidence requirement: retain the closing commit and environment, exact commands and counts, an acceptance-to-proof mapping, every changed-test rationale, and zero unexpected skipped, ignored, or filtered tests.
- Immediate implementation focus: define stable namespaced identities and versioned Manifest/transition descriptor contracts, prove transitions cannot masquerade as deployable releases/instances/providers, then add Core persistence and discovery APIs.

## 2026-07-13 - Sprint 5A Final Closeout

- Completed:
  - accepted the reviewed Dashboard directory, detail, editor, and viewer UI; retained the established outer route panels while giving charts and Tables subdued half-strength gradient surfaces and preserving configured Stat Card fills
  - completed native Dashboard composition over stable `ComponentVersion` ids, including atomic full-layout reconciliation, typed placement config, deterministic reflow, granular sizing through row 240, per-kind minimums, redacted placeholders, total-placement counts, and current-published update-in-place compatibility guards
  - consolidated editor mutation gating under one operation state, added direction-aware history restoration, bounded viewer polling with explicit success/retryable/terminal outcomes, and selected one deterministic comparator from each Table column's declared data type
  - decomposed Dashboard API and viewer/editor responsibilities into focused domain, service, repository, request, projection, reconciliation, and presentation modules
  - consolidated policy-neutral web transport plus shared side-sheet, modal, search, placement-editor, Table toolbar, column-selector, pagination, fullscreen, and visibility-scope UI primitives across Dashboard and adjacent application surfaces
  - retained owner-bound RAII cleanup for the current drag/resize event path and carried a focused Pointer Events plus `setPointerCapture` migration into the durable roadmap backlog with capture-loss, accessibility, modal, touch, and pen regression requirements
  - corrected the existing-database migration preflight for placement count, geometry overlap, fallback-row exhaustion, and mutable current-published Component kinds; cleaned the seed to one nine-placement Dashboard inside the 240-row/240-placement contract
- Validation:
  - `scripts/local-launch.ps1` rebuilt the production CSS, split WASM, and API image, recreated only the Sprint 5A Compose stack without deleting the reviewed database, and returned healthy application routes at `http://localhost:8080`; after disposable smoke teardown, `scripts/local-launch.ps1 -SkipBuild` recreated and seeded the final user-testable deployment
  - `scripts/uat-sprint.ps1 -BaseUrl "http://localhost:8080"` passed organization, Forms, Datasets, Components, Dashboards, and seed flows
  - the first exact `scripts/smoke.ps1` closeout run exposed an empty optional cookie-path bug in its `curl` argument construction after all six demo-flow tests passed; the helper now rejects null/blank paths, parses cleanly, and the exact rerun passed with 52 Dataset rows, 50 Component rows, 26 seeded visual points, and 20 bounded visual points
  - `cargo test -p tessara-api` passed 90 unit tests, the published-version Dashboard compatibility integration, 14 Dashboard composition tests, six demo-flow tests, 25 workflow-runtime tests, and documentation tests
  - `cargo test -p tessara-web` passed 11 root route, document, SSR-bootstrap, and hydration tests after compiling the complete feature/UI graph
  - `npx playwright test` passed 50/50 application scenarios, including all eight Dashboard composition/viewer scenarios and constrained Dashboard/Component visibility coverage
  - `cargo fmt --all` completed successfully; the preceding post-review full all-feature Rust workspace matrix, strict Clippy, native/WASM dependency checks, production build, and web-crate boundary audit also passed
- Next Sprint: Sprint 5B Scoped Analytics And Presentation Hardening Slice

### Sprint Handoff / Demo Instructions

#### Dashboard Directory, Detail, And Visibility

- Role: admin
- Paths:
  - `http://localhost:8080/dashboards`
  - `http://localhost:8080/dashboards/{dashboard_id}` (follow `Demo Operations Dashboard` from the directory)
- Steps:
  1. Sign in as `admin@tessara.local` and open the Dashboard directory.
  2. Search for `Demo Operations Dashboard`; confirm the row remains aligned and exposes named View/Edit icon actions.
  3. Open its detail route and use the visibility-node count link to open, search, and follow a node from the shared side sheet.
- Expected:
  - the directory and detail surfaces reuse established Tessara search, table, stat-card, action, and disclosure patterns; the Dashboard reports nine total placements without exposing expanded node metadata by default
- Acceptance check:
  - pass when the Dashboard is searchable, all actions remain capability-aware and accessible, and the visibility sheet lists linked nodes without page overflow
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `end2end/tests/permissions.spec.ts`, `docs/audits/sprint-5a-ui-review-2026-07-13`, and the final closeout Playwright/UAT output

#### Dashboard Composition And Stable Version Binding

- Role: admin
- Paths:
  - `http://localhost:8080/dashboards/{dashboard_id}/edit` (use the seeded Dashboard's Edit action)
- Steps:
  1. Open Components, use the shared search and kind picker, and add a published Component version.
  2. Move and resize the placement with pointer and direct controls, including a height above six rows; inspect its exact pinned version and use `Preview selected`.
  3. Close the nested preview and Placement details sheet, save the layout, reload it, then remove the test placement and save again.
  4. Confirm `Preview Dashboard` remains disabled while unsaved changes exist and becomes available after a successful save.
- Expected:
  - one coordinated operation state prevents overlapping saves/settings mutations; metadata-only composition makes no unsolicited Component execution requests; successful saves retain stable placement ids and exact version bindings
- Acceptance check:
  - pass when add, move, resize, preview, save, reload, and remove preserve canonical non-overlapping geometry and dirty-state rules
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `crates/tessara-api/tests/dashboard_composition.rs`, `crates/tessara-api/tests/component_dashboard_compatibility.rs`, and the final closeout API/web test output

#### Dashboard Viewer And Standard Component Presentation

- Role: admin
- Paths:
  - `http://localhost:8080/dashboards/{dashboard_id}/view` (use the seeded Dashboard's View action)
- Steps:
  1. Confirm Table, Bar, Line, Pie, Donut, and Stat Card placements render from their exact pinned versions.
  2. Page and resize a Table result, use Reset and Columns, then open the standard Fullscreen icon and continue paging without resetting state.
  3. Close fullscreen with Escape and confirm focus returns to its trigger.
  4. Narrow the viewport and confirm placements stack in reading order, controls remain reachable, and no horizontal overflow appears.
- Expected:
  - charts and Tables use the approved rounded half-strength gradient without hard borders; Stat Cards retain their configured fill; Tables remain complete and server-paged; bounded polling and six-request scheduling isolate retryable or terminal placement failures
- Acceptance check:
  - pass when every supported kind renders, Table state survives fullscreen and paging, one failed placement cannot block siblings, and the viewer remains responsive
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `crates/tessara-web-component-viewer`, `docs/audits/sprint-5a-ui-review-2026-07-13`, and the final closeout browser output

#### Scoped And Redacted Dashboard Access

- Role: scoped operator and manage-without-read test accounts provisioned by the permissions suites
- Paths:
  - `http://localhost:8080/dashboards`
  - `/dashboards/{dashboard_id}/edit`
  - `/dashboards/{dashboard_id}/view`
- Steps:
  1. As a scoped reader, open an in-scope Dashboard containing a Component outside the caller's Component visibility.
  2. Confirm the hidden placement keeps its saved footprint and contributes to the total count while exposing no title, Component, version, Dataset, or execution metadata.
  3. As a manage-without-read operator, direct-load the editor and confirm management is available while the reader directory remains denied.
- Expected:
  - Dashboard and Component scope are enforced independently; hidden placements are generic redacted placeholders and management authority remains narrowed to the editor
- Acceptance check:
  - pass when allowed scoped operations succeed and out-of-scope metadata, execution, and directory access remain unavailable
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `end2end/tests/permissions.spec.ts`, `docs/playwright-permissions-scenarios.md`, and Dashboard split-capability API tests

#### Native SSR, Capacity, And Shared-UI Regression

- Role: admin plus the constrained permission fixtures
- Paths:
  - all Dashboard application routes
  - `http://localhost:8080/forms`
  - `http://localhost:8080/workflows`
- Steps:
  1. Disable JavaScript and direct-load Dashboard directory, detail, editor, and viewer routes; confirm useful safe HTML rather than loading-only shells.
  2. Exercise Dashboard, Form, and Workflow side sheets and nested dialogs; confirm Escape, Tab containment, background inertness, and opener focus restoration.
  3. Review the capacity runbook and the disposable 241st-placement/preflight test evidence rather than mutating the seeded Dashboard.
- Expected:
  - native routes remain hydration- and bridge-free; shared disclosures behave consistently across application surfaces; over-cap, overlapping, or fallback-exhausted data is rejected non-destructively before serving
- Acceptance check:
  - pass when SSR exposes no hidden metadata, shared UI remains keyboard-operable, and the migration/capacity tests leave saved data unchanged on failure
- Evidence location:
  - `crates/tessara-api/tests/dashboard_ssr.rs`, `crates/tessara-api/tests/dashboard_composition.rs`, `crates/tessara-web-ui`, `end2end/tests`, `docs/sprints/sprint-5a-dashboard-capacity-runbook.md`, and the final smoke/UAT output

### Acceptance Mapping

- Exit condition: a tester can assemble and view Dashboards through the app.
  - Plan criteria covered: searchable directory and total counts; read denial; create; manage-without-read editing; metadata/scope editing and atomic incompatible-scope rollback; separate detail, editor, and viewer flows.
  - Manual demonstration: Dashboard Directory, Detail, And Visibility; Dashboard Composition And Stable Version Binding; Dashboard Viewer And Standard Component Presentation.
  - Automated check: `npx playwright test`, `cargo test -p tessara-api`, `cargo test -p tessara-web`, and `scripts/uat-sprint.ps1`.
- Exit condition: Dashboard composition depends on exact `ComponentVersion` ids, supports every delivered Component kind, and preserves stable-id current-published update-in-place behavior without repinning newer versions.
  - Plan criteria covered: published/superseded candidate selection; Dashboard-manage plus Component-read authorization without Component-manage; draft rejection; legacy fallback and typed seed geometry; granular sizes and kind minimums; pointer/keyboard/direct reflow and invalid-target rejection; atomic full-layout save; stable placement ids; redacted-row retain/move/resize/remove; metadata-only editing and dirty-preview behavior.
  - Manual demonstration: Dashboard Composition And Stable Version Binding; Dashboard Viewer And Standard Component Presentation.
  - Automated check: explicit-version request assertions in `end2end/tests/dashboards.spec.ts` and compatibility/lifecycle assertions in `crates/tessara-api/tests/component_dashboard_compatibility.rs` and `crates/tessara-api/tests/dashboard_composition.rs`.
- Exit condition: Dashboard viewer and composition boundaries preserve scoped Dashboard, Component, and Dataset visibility.
  - Plan criteria covered: generic geometry-preserving redaction with unchanged total counts; reader-only action/draft hiding; scoped list/load/place/execute denial; manage-without-read positive access; hidden binding non-disclosure.
  - Manual demonstration: Scoped And Redacted Dashboard Access.
  - Automated check: Dashboard split-capability/redaction API tests plus `end2end/tests/dashboards.spec.ts` and `end2end/tests/permissions.spec.ts`.
- Exit condition: touched Dashboard routes remain native SSR and do not revive product-facing bridge logic.
  - Plan criteria covered: `ComponentVersion`-only product contracts; useful JavaScript-disabled metadata/layout; hydration-, console-, and bridge-request guards.
  - Manual demonstration: Native SSR, Capacity, And Shared-UI Regression.
  - Automated check: `cargo test -p tessara-api --test dashboard_ssr` plus Playwright JavaScript-disabled, console, and request guards.
- Exit condition: storage and display layout remain valid at the 240-placement/240-row boundary without truncating or silently repairing incompatible existing data.
  - Plan criteria covered: 240-placement rejection; all remaining-row heights; cross-row-240 reflow rejection; non-destructive count/overlap/fallback-exhaustion preflight; complete paged Tables; near-viewport lazy execution; version labels; failure isolation; current-published mutation and separate-version non-repinning.
  - Manual demonstration: Native SSR, Capacity, And Shared-UI Regression.
  - Automated check: migration preflight, exactly-240/241st-placement, overlap, fallback-exhaustion, and rollback assertions in `crates/tessara-api/tests/dashboard_composition.rs`, plus `scripts/smoke.ps1`.
- Exit condition: application-wide UI reuse does not regress existing authoring/disclosure surfaces touched by consolidation.
  - Manual demonstration: Native SSR, Capacity, And Shared-UI Regression.
  - Automated check: shared-UI, Forms, Workflow, Component, Dashboard, full Playwright, and web-crate boundary checks.

## 2026-07-13 - Sprint 5A Shared UI And Deployed-UI Follow-up

- Completed:
  - revised the Dashboard editor to keep the composition canvas primary, with Components in a shared left side sheet and selected Placement details in a shared right side sheet
  - retained granular placement sizing while enforcing the code-defined Table minimum of `6 x 4`; repacked the seeded Session Log Table to `12 x 6` and kept all nine seeded placements within row 20 and the 240-placement/row storage contract
  - removed the viewer grid treatment behind cards, kept embedded Tables complete and server-paged, tightened Table density, and replaced Table geometry counts with a state-preserving `View fullscreen` action
  - aligned the Dashboard directory with Forms and Components through the shared page header, search, semantic table/card, count disclosure, compact actions, and pagination patterns; corrected desktop action overflow and phone description wrapping
  - introduced shared `SideSheet`, `ModalDialog`, and `FullscreenDialog` lifecycle ownership for Portal placement, Escape/Tab behavior, nested dialogs, background inertness, body scroll locking, and opener focus restoration, including conditionally unmounted disclosures
  - migrated the Dashboard selected preview, Forms attached-node sheet, and Workflow available-node/assigned-user sheets to the shared dialog/search primitives; made Dashboard, Forms, and Workflow disclosure triggers expose stable dialog relationships and reactive expanded state
  - made `InteractiveDataTable` compose shared `TableSearch` and `TablePaginationFooter`, compacted the shared row-count selector, and removed the orphaned root-local searchable-table implementation
  - hid noninteractive grid-guide cells from the accessibility tree while preserving their pointer-target data, eliminating hundreds of decorative row/column announcements in the editor
  - captured the desktop, tablet, and phone audit with comparison screenshots from established Forms and Components surfaces in `docs/audits/sprint-5a-ui-review-2026-07-13`
  - completed the annotation follow-up: replaced Table full-screen text with the shared Fullscreen icon and tooltip semantics; removed redundant viewer placement counts, geometry labels, and Table summary chrome; added a capability-gated `Edit Dashboard` action
  - condensed Dashboard detail metadata into metric cards and replaced the expanded Visibility value with a searchable shared side sheet containing live counts and links to all in-scope nodes; removed the invariant 12-column layout row
  - aligned the editor palette with Components by composing shared icon-bearing search and the established compact kind picker, including a no-overflow 390-pixel-width check
  - implemented the approved viewer presentation split while retaining the established outer route panel: charts and Tables use one subdued rounded soft-surface token without hard borders or shadows, charts use in-chart titles, and Stat Cards render their configured semantic fill without outer placement chrome
  - moved optional Table title and full-screen behavior into the route-free standard Component Table renderer; Dashboard placements now pass only stable presentation ids/text, while the renderer owns the shared query/paging state, puts Reset -> Columns -> Fullscreen in the standard toolbar, and prevents nested full-screen triggers
  - removed the arbitrary six-row placement-height ceiling; placements may use all remaining rows through row 240 while per-kind minimums, collision rules, and the 240-row/240-placement contracts remain enforced
  - corrected repair-mode resizing so a malformed fallback may transition to valid kind-conforming geometry without weakening normal resize validation, and kept Stat Cards compact at phone widths instead of inheriting the chart/Table minimum height
  - corrected the final surface annotation after clarification: removed the intermediate neutral overlay and reduced the original shared blue-to-teal gradient to 50% of its prior visual strength for all chart placements and standard/embedded Tables; chart marks, labels, Table controls, and rows remain fully opaque, while Stat Cards retain their configured fill
  - corrected the Dashboard directory action-row annotation by keeping the desktop Actions cell in native table layout, nesting View/Edit in a reusable icon-action group with Eye/Pencil icons and named tooltips, narrowing the action column, and preserving the mobile card's labeled action row
- Validation:
  - affected all-feature native checks, the root WASM hydration graph, strict Clippy, Tailwind 4.2.4 compilation, formatting, and diff checks passed
  - 67 focused shared-UI/Forms/Workflow/Dataset tests passed; final focused core, Dashboard-domain, Dashboard-web, and Component-viewer suites passed 23/23, 24/24, 25/25, and 18/18
  - all 12 database-backed Dashboard composition/API tests passed against a disposable isolated database, including the 240-row boundary and full-height Table case
  - the dedicated saved-viewer Table/chart/Stat E2E contract scenario passes Playwright discovery and TypeScript transformation as part of the eight-scenario Dashboard file
  - the navigation/lazy-render Table-fullscreen regression passed repeatedly and covers desktop open/close, client navigation away/back, phone reflow, lazy Table rendering, dialog mounting, and expanded-state transitions
  - in-app browser review exercised desktop, tablet, and phone directory/editor/viewer states, side-sheet and full-screen dialog semantics, complete embedded paging, shared query state, keyboard close behavior, capability-gated viewer actions, Visibility filtering/linking, compact editor filters, and console output
  - the final release image was rebuilt and health-checked with only the Sprint 5A API and Postgres containers running; deployed review at 1951 x 806, 1440 x 1000, and 390 x 844 confirmed the approved soft surfaces, wide and wrapped Table toolbar layouts, desktop/phone full-screen behavior, focus restoration, popover hit-testing, paging continuity, and zero page-level horizontal overflow while the user-directed outer route panels remain intact
  - the clarified half-gradient release follow-up was rebuilt and health-checked, then verified in the in-app browser at 1168 x 912 in dark and light themes and at 390 x 844 in dark mode; all four chart kinds and mounted standard/embedded Table surfaces resolved to the same half-strength gradient with element opacity `1`, page-level horizontal overflow remained zero, and Stat Card fill remained independent
  - the Dashboard action-row release follow-up was rebuilt and health-checked; 25 Dashboard crate tests and the updated native-routes Playwright scenario passed, the eight-scenario Dashboard file passed discovery/transformation, and deployed checks at 1262 x 912 light/dark plus 390 x 844 mobile confirmed full row/cell alignment, icon contrast, accessible names/tooltips, and zero horizontal overflow
- Follow-up disposition:
  - shared Table toolbar, selector, pagination, fullscreen, modal, side-sheet, and search presentation is resolved for Sprint 5A; client-backed and server-backed state engines remain deliberately separate
  - Dashboard API, editor, request, visual, reconciliation, repository, and viewer responsibilities received the planned focused decomposition
  - remaining hand-rolled disclosure candidates outside the touched Dashboard/Form/Workflow surfaces are optional future consolidation work rather than an unresolved Sprint 5A decision
  - a Pointer Events plus `setPointerCapture` migration remains a focused future-work item in `docs/roadmap.md`; owner-bound RAII cleanup is the accepted current implementation

## 2026-07-12 - Sprint 5A Dashboard Composition Closeout

- Completed:
  - delivered native Dashboard directory, create, detail, composition-editor, and focused-viewer routes over exact `ComponentVersion` bindings
  - delivered atomic full-layout reconciliation with stable placement ids, deterministic reflow, direct/pointer/keyboard movement and sizing, redacted placeholder preservation, and the 240-placement/240-row storage contract
  - extracted reusable grid policy into `tessara-core`, Dashboard composition policy into `tessara-dashboards`, shared placement interactions into `tessara-web-ui`, Dashboard routes into `tessara-web-dashboards`, and the route-free exact-version renderer into `tessara-web-component-viewer`
  - delivered full server-paged embedded Tables, viewport-lazy viewer mounting with six-request execution concurrency, bounded renderer/Table-state caches, and isolated placement failure states
  - documented stable-id current-published update-in-place behavior, immutable superseded versions, non-repinning newer versions, seeded typed V1 geometry, and the over-cap deployment preflight/runbook
- Validation:
  - closeout was first validated against an isolated optimized release at port 8081 so the pre-existing Sprint 4B stack was not disturbed; after deployment authorization, Sprint 4B was brought down without deleting its volume and a fresh Sprint 5A Compose deployment was built, migrated, seeded, and health-checked on the standard `http://127.0.0.1:8080` and `tessara` database
  - the fresh database applied migration `1:baseline` and `2:dashboard placement capacity`, then seeded five demo accounts, 20 nodes, nine Components, one Demo Operations Dashboard, and nine typed V1 placements within row 20; the content-heavy Session Log Table occupies `12 x 6`
  - `cargo leptos build --release --split` plus `npm run tailwind:build` - passed; the release CSS and JavaScript assets return 200 without unresolved Tailwind imports
  - `.\scripts\validate.ps1` with `TEST_DATABASE_URL` - passed in 20m14s: format, native/SSR/wasm32 checks, 11 web tests, 87 API unit tests, 11 Dashboard composition tests, one Dashboard SSR test, six demo-flow tests, and 25 workflow-runtime tests
  - `cargo check --workspace --all-features` - passed
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed
  - `cargo test --workspace --all-features` - passed, including API, composition, migration, SSR, exact-version viewer, scheduler, Form characterization, and documentation tests
  - exact `cargo test -p tessara-web`, `cargo test -p tessara-api`, and `cargo test -p tessara-web-ui --all-features` invocations - passed; the shared UI set includes the grid-guide hydration-markup regression
  - real `wasm32-unknown-unknown` hydrate checks for `tessara-web`, `tessara-web-dashboards`, and `tessara-web-component-viewer` - passed
  - `npm test` and `.\scripts\validate-e2e.ps1 -BaseUrl http://127.0.0.1:8081` - passed: 48/48 browser tests each, including all six Sprint 5A Dashboard scenarios and scoped/redacted permission coverage
  - `.\scripts\smoke.ps1 -UseExistingService -BaseUrl http://127.0.0.1:8081 -KeepServices` - passed with 52 Dataset rows, 50 Component rows, 26 seeded visual points, 20 bounded visual points, and the nine-placement Dashboard contract
  - `.\scripts\uat-sprint.ps1 -BaseUrl http://127.0.0.1:8081` - passed for organization, Forms, Datasets, Components, Dashboards, exact-version execution, bounded Table paging, native SSR markers, and seed flows
  - in-app browser visual QA - passed at desktop, tablet, and phone widths after the follow-up review: a canvas-first editor with shared Component and Placement-details side sheets, nine readable symbolic placements, a solid viewer surface, complete paged embedded Tables, and a state-preserving full-screen Table treatment
  - `cargo audit --quiet` - passed with three allowed transitive warnings (`paste`, `proc-macro-error2`, and `anyhow`); dependency boundaries, PowerShell parsing, `cargo fmt --all -- --check`, and `git diff --check` - passed
- Next Sprint: Sprint 5B Scoped Analytics And Presentation Hardening Slice

### Sprint Handoff / Demo Instructions

#### Dashboard Directory, Creation, And Detail

- Role: admin
- Paths:
  - `http://127.0.0.1:8080/dashboards`
  - `http://127.0.0.1:8080/dashboards/new`
  - `http://127.0.0.1:8080/dashboards/{dashboard_id}` (follow the Demo Operations Dashboard link from the directory)
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Search the Dashboard directory for `Demo Operations` and confirm its total placement count.
  3. Create an in-scope Dashboard with a name, optional description, and visibility node, then open its detail page.
- Expected:
  - native pages expose capability-appropriate actions, useful metadata, and clear populated or empty states
- Acceptance check:
  - pass when the seeded Dashboard is searchable, the count includes all stored placements, and a scoped create opens a separate detail surface
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `crates/tessara-api/tests/dashboard_composition.rs`, and the closeout Playwright/smoke transcripts

#### Symbolic Dashboard Composition

- Role: admin
- Paths:
  - `http://127.0.0.1:8080/dashboards/{dashboard_id}/edit` (use the seeded Dashboard id from the directory)
- Steps:
  1. Add a published Component version from the metadata-only palette.
  2. Drag it into occupied geometry and confirm deterministic reflow; resize it with a handle and direct Width/Height controls.
  3. Edit its title, inspect the exact-version link, and use `Preview selected`.
  4. Close the preview, save once, reload, remove the placement, and save again.
- Expected:
  - editor tiles remain symbolic until explicit preview; valid edits preserve non-overlapping geometry and stable ids, invalid inputs restore the last canonical values, and save reconciles the full layout atomically
- Acceptance check:
  - pass when add/reflow/resize/title/preview/save/remove survive reload without unsolicited Component execution
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `crates/tessara-core/src/grid_layout.rs`, `crates/tessara-api/tests/dashboard_composition.rs`, and `design-qa.md`

#### Exact-Version Dashboard Viewer And Embedded Table

- Role: admin
- Paths:
  - `http://127.0.0.1:8080/dashboards/{dashboard_id}/view` (use the seeded Dashboard id from the directory)
- Steps:
  1. Confirm Table, Bar, Line, Pie, Donut, and Stat Card placements render in saved geometry.
  2. In the Session Log Table, use Next and change the page size; confirm the tile remains bounded while later rows are reachable.
  3. Use the standard Table toolbar's Fullscreen icon; confirm the same paging state opens in the dialog, no nested Fullscreen action appears, and Escape returns focus to the trigger.
  4. Narrow the viewport below 780px and confirm cards stack in deterministic reading order while Table controls wrap without page overflow.
- Expected:
  - each available placement executes its pinned exact version; charts and Tables use the approved subdued borderless surfaces, Stat Cards keep configured fills, and the standard Table presentation retains complete server-backed paging while viewer execution stays responsive and bounded
- Acceptance check:
  - pass when all six kinds render, Table controls issue exact-version page requests, and the narrow layout has no horizontal overflow
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `crates/tessara-web-component-viewer`, and the closeout Playwright/UAT transcripts

#### Scoped And Redacted Dashboard Access

- Role: scoped operator fixtures from the permissions and Dashboard suites
- Paths:
  - `/dashboards`
  - `/dashboards/{dashboard_id}/edit`
  - `/dashboards/{dashboard_id}/view`
- Steps:
  1. As a scoped reader, open an in-scope Dashboard containing a Component outside the caller's Component visibility.
  2. Confirm the hidden placement keeps its footprint and contributes to the total count but exposes no title, Component, version, or Dataset metadata and issues no execution request.
  3. As a manage-without-read operator, direct-load an in-scope editor and confirm the reader directory remains unavailable.
- Expected:
  - Dashboard and Component scope are enforced independently; management is narrowed to the editor and hidden bindings remain opaque
- Acceptance check:
  - pass when positive scoped operations work and out-of-scope metadata, directory access, and execution remain unavailable
- Evidence location:
  - `end2end/tests/dashboards.spec.ts`, `end2end/tests/permissions.spec.ts`, `docs/playwright-permissions-scenarios.md`, and API split-capability tests

#### Native SSR And Capacity Safety

- Role: admin and scoped operator fixtures
- Paths:
  - all five Dashboard application routes
  - `/api/admin/dashboards/{dashboard_id}/composition`
- Steps:
  1. Disable JavaScript and direct-load Dashboard directory, detail, editor, and viewer routes.
  2. Confirm useful metadata, saved geometry, capability-aware actions, and generic redacted placeholders are present in HTML.
  3. Review the capacity runbook, then attempt a 241st placement against a disposable fixture.
- Expected:
  - native SSR remains useful without hydration or `/bridge/*`; the 241st placement and geometry beyond row 240 fail atomically with stable errors
- Acceptance check:
  - pass when SSR contains no hidden metadata and capacity failures leave the stored composition unchanged
- Evidence location:
  - `crates/tessara-api/tests/dashboard_ssr.rs`, `crates/tessara-api/tests/dashboard_composition.rs`, `end2end/tests/dashboards.spec.ts`, and `docs/sprints/sprint-5a-dashboard-capacity-runbook.md`

### Acceptance Mapping

- Exit condition: a tester can assemble and view Dashboards through the app.
  - Manual demonstration: Dashboard Directory, Creation, And Detail; Symbolic Dashboard Composition; Exact-Version Dashboard Viewer And Embedded Table.
  - Automated check: `npm --prefix .\end2end test`, `cargo test -p tessara-api`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8080"`.
- Exit condition: Dashboard composition depends on exact `ComponentVersion` ids and supports all delivered Component kinds without legacy report/chart/workbench behavior.
  - Manual demonstration: Exact-Version Dashboard Viewer And Embedded Table.
  - Automated check: exact-version request assertions in `end2end/tests/dashboards.spec.ts` and lifecycle assertions in `crates/tessara-api/tests/dashboard_composition.rs`.
- Exit condition: Dashboard viewer and composition endpoints preserve scoped Dashboard and Component visibility.
  - Manual demonstration: Scoped And Redacted Dashboard Access.
  - Automated check: split-capability/redaction API tests plus the Dashboard and permissions Playwright suites.
- Exit condition: touched Dashboard routes remain native SSR and do not revive product-facing bridge logic.
  - Manual demonstration: Native SSR And Capacity Safety.
  - Automated check: `cargo test -p tessara-api --test dashboard_ssr` and Playwright console/request guards for all Dashboard routes.
- Exit condition: storage and layout remain valid at the supported boundary.
  - Manual demonstration: Native SSR And Capacity Safety.
  - Automated check: migration preflight/trigger and exactly-240/241st-placement assertions in `crates/tessara-api/tests/dashboard_composition.rs`, plus `.\scripts\smoke.ps1`.

## 2026-07-12 - Sprint 5A Dashboard Composition Kickoff

- Sprint: `Sprint 5A: Dashboard Composition Slice`, selected from the sole roadmap heading marked `(Next)`.
- Kickoff status: prepared from clean, synchronized `main` at `a09a17c516dd7773fcec700e2d2415c91757ae62`; the UI direction is approved, implementation is unstarted, and application changes await a separate go-ahead.
- Branch: `codex/sprint-5a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-5a`
- Plan file: `docs/sprints/sprint-5a-plan.md`
- Design gate: satisfied on 2026-07-12 for the composition editor only, with the approved Symbolic Builder reference at `docs/mockups/sprint-5a-dashboard-symbolic-builder-approved.png` and decision record at `docs/sprints/sprint-5a-dashboard-editor-design.md`; detail/viewer surfaces follow existing native patterns, and mockup-only Dashboard status/grid-setting controls are illustrative.
- Layout direction: reuse and generalize the Form builder 12-column placement model, including equivalent pointer/keyboard/direct occupied-target movement reflow and collision-rejecting size changes, rather than introducing a separate ordered-list editor.
- Sizing decision: retain every integer width 1 through 12 and every height that fits through row 240 when it satisfies the Component kind's minimum. Table ships with a `6 x 4` minimum, not a fixed size, and may grow to width 12 and through every row remaining below its starting row; other kinds retain the `1 x 1` hard minimum. Use non-binding add defaults of Table `6 x 4`, Bar `6 x 3`, Line `6 x 2`, Pie/Donut `3 x 3`, and Stat Card `3 x 2`; retain code-defined per-kind minimum support for later explicit policy changes. Desktop tracks use a shared, non-user-configurable square-cell rule clamped from 48px through 80px and pointer math measures the rendered track.
- Mockup refinement: keep the editor canvas iconographic and metadata-only, with selected/full Component rendering available only through explicit lazy preview actions. The post-implementation review moved Components and Placement details from persistent columns into shared side sheets so the canvas remains primary.
- Shared ownership direction: extract framework-free grid policy into `tessara-core`, genuinely reusable low-level Leptos placement interactions into `tessara-web-ui`, and Dashboard composition policy into `tessara-dashboards`; preserve the Form builder's current add/config UX and complete the required proposal before extracting `tessara-web-dashboards`.
- Pinning decision: Dashboard placement pins a stable `component_version_id`; an intentional Sprint 4A current-published update-in-place may change the rendered payload under that id, while publishing a separate version never repins automatically.
- Visibility decision: detail/viewer responses preserve every stored placement; inaccessible placements become geometry-preserving redacted placeholders with no hidden Component/version/Dataset metadata, and directory/detail counts report total distinct stored placements.
- Table viewer decision: embedded Table placements use the shared controlled, server-backed complete paged Table renderer and normal viewer affordances; pagination bounds the tile without replacing the Table with a truncated or client-only summary.
- Capacity and seed direction: Dashboards store at most 240 placements and no placement may extend beyond row 240. There is no overflow repair list and a 241st placement is rejected atomically. A non-mutating deployment preflight blocks an over-cap existing database and reports Dashboard ids/counts for an operator-controlled cleanup procedure with backup/export safeguards. Current Demo Operations Dashboard placements receive explicit typed V1 geometry and are repacked or removed if needed to meet the cap, while title-only/empty existing configs remain readable through deterministic position-derived fallback for an in-cap Dashboard. The focused viewer uses viewport-lazy mounting and a named code-level execution-concurrency ceiling below the supported storage cap.
- Planned verification:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --all-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-features`
  - `cargo audit --quiet`
  - `.\scripts\check-web-crate-boundaries.ps1`
  - `.\scripts\validate.ps1`
  - `.\scripts\local-launch.ps1 -FreshData`
  - `npm --prefix .\end2end test`
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
  - `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `git diff --check`
- Immediate implementation focus after approval: protect Dashboard scope/redaction/total-count behavior and current Form placement behavior with targeted regression coverage; complete the focused frontend ownership proposal; then replace the `/dashboards` placeholder with a native, capability-aware directory as the first user-visible vertical slice.

## 2026-07-12 - Sprint 4B Chart And Stat Component Closeout

- Completed:
  - delivered first-class Bar, Line, Pie, Donut, and Stat Card authoring, validation, preview, versioning, publishing, and viewing on the canonical `Dataset -> ComponentVersion` path
  - delivered kind-specific execution endpoints, D3-backed chart rendering, responsive editor behavior, category/series labels and colors, filters, sorting, limits, axis controls, and seeded examples for every Component kind
  - preserved Table Component behavior and scoped Component/Dataset visibility while adding visual kinds
  - decomposed the Component API runtime and frontend editor into focused runtime, editor, typed-config, and test modules
  - bundled D3 locally with licensing attribution and retained native Leptos route ownership without `/bridge/*` assets
- Closeout validation:
  - `crates/tessara-api/migrations` contains only `001_baseline.sql`; a fresh database reports only migration `1:baseline:true` and the six supported Component enum values
  - `.\scripts\local-launch.ps1 -FreshData` - passed: rebuilt the release image, recreated the Postgres volume, applied the squashed baseline, and seeded 58 submitted Demo Session Log responses, four Datasets, nine Components, and one Dashboard
  - `cargo fmt --all -- --check` - passed
  - `cargo check --workspace --all-features` - passed
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed
  - `cargo test --workspace --all-features` - passed: all unit, integration, and doc tests, including 86 API tests, 6 demo-flow tests, 25 workflow-runtime tests, and 22 Component editor tests
  - `npx playwright test` - passed: 41/41 browser tests
  - `.\scripts\validate-e2e.ps1 -BaseUrl http://127.0.0.1:8080` - passed: 41/41 browser tests through the repository validation wrapper
  - `.\scripts\smoke.ps1 -UseExistingService -BaseUrl http://127.0.0.1:8080 -KeepServices` - passed with 52 Dataset rows, 50 Component rows, 26 seeded visual points, and 20 bounded visual points
  - `.\scripts\uat-sprint.ps1 -BaseUrl http://localhost:8080` - passed for organization, forms, Datasets, Components, and seed flows
  - `git diff --check`, D3 renderer JavaScript syntax, and all PowerShell script parser checks - passed
  - hardened browser diagnostics to retain failed page-response URLs and replaced a hydration-sensitive one-shot mobile geometry read with a live polling assertion
- Legacy visual-analysis endpoint inventory:
  - no legacy visual-analysis endpoints were touched; all Sprint 4B execution and metadata behavior is owned by Component and ComponentVersion routes
- Next Sprint: Sprint 5A Dashboard Composition Slice

### Sprint Handoff / Demo Instructions

#### Visual Component Authoring And Preview

- Role: admin
- Paths:
  - `http://localhost:8080/components/new`
  - `http://localhost:8080/components/demo-session-log-bar/edit`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open Create Component and choose the Demo Session Log Dataset version from the searchable Dataset Version picker.
  3. Select each visual Component kind and review its stable kind-specific configuration panel.
  4. For Bar, select Session Date as Category, Completed As Planned as Series, Participants as Value, and review horizontal/vertical plus grouped/stacked preview behavior.
  5. Add a filter and verify the preview updates from at most 100 source rows.
- Expected:
  - the editor exposes only applicable controls, validation is announced in place, help buttons provide on-click guidance, and the sticky desktop preview or mobile preview drawer renders the current draft without saving
- Acceptance check:
  - pass when Bar, Line, Pie, Donut, and Stat Card drafts each produce a valid preview and no control or panel escapes its container
- Evidence location:
  - `end2end/tests/components.spec.ts` visual authoring and mobile preview scenarios

#### Publish, Version, And View Visual Components

- Role: admin
- Paths:
  - `http://localhost:8080/components/demo-session-log-bar`
  - `http://localhost:8080/components/demo-session-log-bar/versions`
  - `http://localhost:8080/components/demo-session-log-line`
  - `http://localhost:8080/components/demo-session-completion-pie`
  - `http://localhost:8080/components/demo-session-completion-donut`
  - `http://localhost:8080/components/demo-session-total-participants-stat-card`
- Steps:
  1. Open each seeded visual Component viewer and confirm the expected chart or stat treatment renders.
  2. Use Edit and Versions from the viewer header.
  3. Save a draft, publish it, and create a later Component version with a version note.
  4. Open an explicit historical version and confirm it renders from that version's Dataset major-line binding.
- Expected:
  - viewers render D3 Bar, Line, Pie, and Donut charts or a bounded Stat Card; version actions remain on the Component workflow and historical versions do not silently resolve to the current version
- Acceptance check:
  - pass when every delivered kind can be published and viewed and explicit-version endpoints return the requested version
- Evidence location:
  - `end2end/tests/components.spec.ts` visual lifecycle, wrong-kind, and historical-version assertions

#### Labels, Colors, Filters, And Responsive Behavior

- Role: admin
- Paths:
  - `http://localhost:8080/components/demo-session-completion-donut/edit`
  - `http://localhost:8080/components/demo-session-log-line/edit`
- Steps:
  1. Change a Category or Series field and verify its labels/colors table refreshes to the new distinct values.
  2. Set a legend title, display labels, and theme colors, then confirm the preview uses them.
  3. Narrow the browser to a mobile viewport and open Preview from the floating action button.
  4. Close the preview drawer and confirm focus returns to the trigger.
- Expected:
  - stale category values do not persist, color menus remain visible and dismiss outside, charts avoid overlapping labels, and the mobile editor remains horizontally contained
- Acceptance check:
  - pass when label/color changes render correctly and keyboard/mobile preview interactions remain usable without horizontal overflow
- Evidence location:
  - `end2end/tests/components.spec.ts` editor containment and accessible preview-drawer scenario

#### Scoped Visual Component Access

- Role: scoped operator fixture used by the permissions suite
- Paths:
  - `/components`
  - `/api/components/{component_ref}/bar`
- Steps:
  1. Run the permissions suite to create visible and hidden Dataset-backed Bar fixtures.
  2. Confirm the scoped user lists, reads, views, and executes the in-scope published Bar.
  3. Confirm the same user cannot list, direct-load, or execute the out-of-scope Bar.
- Expected:
  - published visual metadata and execution follow Dataset and Component scope; hidden assets do not leak names, versions, or chart data
- Acceptance check:
  - pass when positive in-scope access succeeds and every hidden direct request is forbidden or absent
- Evidence location:
  - `end2end/tests/permissions.spec.ts`
  - `docs/playwright-permissions-scenarios.md`

### Acceptance Mapping

- Exit condition: a tester can create a Bar, Line, Pie, Donut, and Stat Card through the application UI.
  - Manual demonstration: Visual Component Authoring And Preview.
  - Automated check: `end2end/tests/components.spec.ts` visual authoring lifecycle.
- Exit condition: visual components validate, save, publish, version, and render from ComponentVersion without deprecated workbench assets.
  - Manual demonstration: Publish, Version, And View Visual Components.
  - Automated check: API Component tests plus Playwright visual lifecycle, explicit-version, wrong-kind, browser-console, and `/bridge/*` request assertions.
- Exit condition: Table endpoints remain table-only while kind-specific visual endpoints return stable view models and reject mismatches.
  - Manual demonstration: publish/view each kind and open historical versions.
  - Automated check: `cargo test -p tessara-api components::tests::` and Playwright wrong-kind route assertions.
- Exit condition: reader and management routes enforce published state, capability, and Dataset/Component scope.
  - Manual demonstration: Scoped Visual Component Access.
  - Automated check: `end2end/tests/permissions.spec.ts` scoped visual Component scenarios.
- Exit condition: touched Component routes remain native, hydration-clean, console-clean, and free of `/bridge/*` assets.
  - Manual demonstration: open editor and viewer routes directly and navigate between Edit, Versions, and viewer actions.
  - Automated check: full Playwright suite console collector and visual-route request listener.
- Exit condition: retained legacy visual-analysis endpoints are adapter-only and scope-safe if touched.
  - Manual demonstration: not applicable; no legacy visual-analysis endpoints were touched.
  - Automated check: route/code inventory confirms Sprint 4B behavior is implemented under Component/ComponentVersion endpoints only.

## 2026-07-11 - Sprint 4B Full Implementation Review

- Completed a full correctness, UX, accessibility, performance, migration, and code-quality review of the Sprint 4B implementation.
- Correctness and upgrade repairs:
  - restored the immutable baseline migration and moved visual component schema changes into forward-only migrations `002` and `003`, with regression tests for baseline integrity and migration ownership
  - scoped editor field catalogs and distinct category/series values to the selected Dataset major line instead of the latest revision/current-row sample
  - moved published visual aggregation into SQL and limited previews to at most 100 source rows before grouping
  - preserved negative Bar/Line values, rejected negative Pie/Donut summaries, made unique counts collision-free, and made numeric/category/comparison sorting type-aware
  - enforced additive-only stacked Bars and returned role-specific validation codes for stale Summary, Category, Comparison, X, filter, and Table fields
- UX, resilience, and accessibility repairs:
  - bundled D3 7.9.0 locally with its ISC notice and build-fingerprinted asset URLs, removing the runtime CDN dependency and stale-cache risk
  - debounced draft previews, added a chart skeleton/failure state, and kept row-count previews valid without a Value field
  - made help content real on-click tooltip content with independently named CircleHelp triggers and stable accessible names for associated form controls
  - added keyboard support to the Dataset Version combobox and focus containment/return behavior to the mobile preview drawer
  - verified the sticky desktop preview and mobile drawer visually; 1440px and 600px probes had no horizontal overflow, the Bar preview rendered 26 SVG bars, and the browser console remained error-free
- Final validation against a fresh rebuilt and reseeded stack:
  - `cargo test --workspace --all-features` - passed: all workspace unit, integration, and doc tests, including 87 API tests, 6 demo-flow tests, 25 workflow-runtime tests, and 22 Component editor tests
  - `cargo fmt --all -- --check` - passed
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed
  - `npx playwright test` - passed: 41/41 browser tests, including visual lifecycle, permissions, historical Dataset-major behavior, mobile containment, and accessible preview-drawer coverage
  - `scripts/smoke.ps1 -UseExistingService -BaseUrl http://127.0.0.1:8080 -KeepServices` - passed
  - `scripts/uat-sprint.ps1 -BaseUrl http://localhost:8080` - passed
  - migration immutability, `git diff --check`, JavaScript syntax, and PowerShell parser checks - passed
- Maintainability remediation:
  - split Component runtime execution, aggregation, filtering, and transforms into `components/runtime.rs`, leaving route lifecycle and authorization composition in `components/mod.rs`
  - split Component editor controls, typed config serialization, and tests into focused `pages/editor.rs`, `pages/editor_config.rs`, and `pages/tests.rs` modules; the config module now declares an explicit dependency boundary instead of inheriting the page module namespace
  - retained Sprint 4B migrations `002` and `003` during development and documented the required baseline squash, migration-test replacement, fresh-volume rebuild, and complete validation sequence for sprint closeout
- Remaining follow-up maintainability debt:
  - Dashboard composition remains outside Sprint 4B; Stat Card CSS is container-bounded, but a real Dashboard-grid integration test should be added when Dashboard layout authoring is implemented

## 2026-07-10 - Sprint 4B Component Editor Alignment

- Reworked the shared Component editor around the implementation guide:
  - added compact Dataset Context and Component Kind panels, desktop kind buttons with a mobile select fallback, and confirmation before clearing incompatible kind-specific settings
  - added a Bar-specific role editor for Category, Series, and Measure plus order, category limit, orientation, comparison layout, format, and axis-title controls
  - added a sticky current-draft preview backed by `POST /api/admin/components/preview`, with validation/loading/error states and a plain-language execution summary
  - added collapsible visual filters and Labels & colors sections; Bar labels/colors now use Comparison Field values and single-series bars use one consistent color
  - moved Bar legends into normal flow above the SVG so they wrap without overlapping the plot
  - added `Count rows` and `Do not summarize`; row counts hide Value field, and unsummarized duplicate groups fail explicitly instead of being silently repaired
  - separated category, series, and measure missing-value policies so each control affects only its data role, with backward-compatible fallback for older saved configs
  - moved keyboard focus into the newly rendered kind editor after a confirmed kind change
  - replaced the generic visual-config switch with dedicated Bar, Line, Pie/Donut, and Stat Card editors plus narrowly shared Measure and Order controls
  - added a typed `ComponentConfigDraft` boundary with per-kind draft structs and serializers; removed the former component-type JSON builder
  - corrected Bar DOM order so collapsed Filters sit between field mapping and Order & display, followed by Labels & colors
  - corrected comparison Bar limiting so Category limit retains whole categories and every series within each retained category; total-value ordering now uses category totals
- Demo and regression coverage:
  - refreshed the Demo Session Participants Bar seed as a stacked comparison chart by Session Date and completion status
  - rebuilt and reseeded the local Docker stack with 58 submitted Demo Session Log responses and nine example Components
  - expanded the visual Component Playwright scenario for draft preview, kind confirmation, comparison display rows, legend placement, calculation disclosure, and multi-kind publish/view workflows
- Validation:
  - `cargo check -p tessara-api -p tessara-web-components --features tessara-web-components/hydrate` - passed
  - `cargo test -p tessara-api components::tests::` - passed: 36/36, including whole-category comparison limit coverage
  - `cargo test -p tessara-web-components pages::tests::` - passed: 18/18, including per-kind typed serialization coverage
  - `npx playwright test tests/components.spec.ts --grep "admin can author, publish, and view visual components"` - passed against the final rebuilt stack: 1/1
  - `npx playwright test tests/components.spec.ts --list` - passed after the final calculation and null-policy additions
  - desktop browser probe confirmed the Bar legend precedes the SVG, has zero overlap, contains completion-status series, and causes no horizontal overflow
  - 390x844 browser probe confirmed a single-column editor, mobile kind select, preview below controls, and no horizontal overflow
  - live Bar DOM probe confirmed Fields & calculation, collapsed Filters, Order & display, and collapsed Labels & colors in guide order
  - live accessibility probe confirmed a real six-item kind radiogroup with one selected radio, polite preview/findings regions, and no separate Validate action
  - save-validation browser coverage confirmed an invalid save remains on the editor, preserves entered identity, and announces Validation Findings before correction
  - `npx playwright test tests/components.spec.ts` - passed against the final rebuilt stack: 2/2 Table and visual Component lifecycle scenarios
  - `git diff --check` - passed with only existing PowerShell LF-to-CRLF warnings

## 2026-07-10 - Sprint 4B Category Display Feedback

- Browser-feedback fix:
  - tightened the Component editor Category Display table so its rows are driven only by the currently selected Category Field values; saved display-label/color entries from a previous Category Field no longer reappear after the field changes
  - guarded asynchronous Category Field value loading so late responses from a previous field or Dataset Version cannot overwrite the current Category Display table
  - defaulted Legend Title to the newly selected Category Field label when authors change the Category Field, without overwriting saved legend text on reopen
  - added a `Versions` action beside `Edit` on Component viewer headers for quicker navigation to version history
  - aligned Component viewer header action spacing with the existing page-header action group spacing
- Validation:
  - `cargo fmt --all -- --check` - passed
  - `cargo check -p tessara-web-components` - passed
  - `.\scripts\local-launch.ps1 -FreshData` - passed after rebuilding and reseeding the local stack
  - `npx playwright test tests/components.spec.ts --grep "visual component"` - passed: 1/1 browser regression covering visual component authoring, Category Field stale-label reset behavior, Legend Title defaulting, and Component viewer version navigation
  - `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices` - passed against the rebuilt seeded stack
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"` - passed: 40/40 browser tests, including component visual authoring/viewing and permission scenarios
  - `cargo test -p tessara-web-components` - passed: 17 component UI helper tests and doc tests
  - `cargo test -p tessara-api` - passed: 75 component/dataset unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed against the rebuilt seeded stack
  - `cargo test -p tessara-web -p tessara-web-datasets -p tessara-web-data-ops` - passed: 5 web shell tests, 24 dataset UI helper tests, 1 data-ops helper test, and doc tests
  - `git diff --check` - passed with only existing PowerShell LF-to-CRLF warnings for `scripts/smoke.ps1` and `scripts/uat-sprint.ps1`

## 2026-07-10 - Sprint 4B Bar Chart Feedback

- Browser-feedback fix:
  - added Bar Visual Config controls for horizontal/vertical orientation and optional X/Y axis labels
  - added a comparison layout control that appears when a Bar component has a Comparison Field, supporting grouped and stacked arrangements
  - carried Bar orientation, comparison layout, and axis labels through the ComponentVersion config, API visual response, seeded demo component, and client DTOs
  - replaced the simple Bar renderer with a D3 axis-based renderer for horizontal/vertical, grouped/stacked bar charts with axis ticks, labels, titles, and comparison legends
- Validation:
  - `cargo check -p tessara-web-components -p tessara-api` - passed
  - `cargo test -p tessara-api components::tests::visual_component_config` - passed: 2/2 component visual config tests
  - `cargo fmt --all -- --check` - passed
  - `node --check crates/tessara-web/assets/tessara-d3-charts.js` - passed
  - `.\scripts\local-launch.ps1 -FreshData` - passed after rebuilding and reseeding the local stack
  - `npx playwright test tests/components.spec.ts --grep "visual component"` - passed: 1/1 browser regression covering the new Bar controls inside visual component authoring

## 2026-07-10 - Sprint 4B Bar Comparison Display Feedback

- Browser-feedback fix:
  - moved Bar comparison legends into a reserved band above the plot area so they no longer overlap the bars
  - changed Bar display overrides so comparison-mode bars use Comparison Field values for legend labels and colors
  - kept summary-only Bar charts on one consistent series color instead of recoloring by Category Field
  - changed the Bar editor Category Display table to load Comparison Field values when a Comparison Field is selected
- Validation:
  - `cargo fmt --all -- --check` - passed
  - `cargo check -p tessara-web-components -p tessara-api` - passed
  - `cargo test -p tessara-api components::tests::` - passed: 32/32 component tests, including Bar comparison label/color mapping
  - `node --check crates/tessara-web/assets/tessara-d3-charts.js` - passed
  - `npx playwright test tests/components.spec.ts --grep "visual component"` - passed: 1/1 browser regression
  - Live Playwright probe on `/components/demo-session-log-bar` and `/edit` confirmed the legend no longer overlaps the first bar and the Bar display table is driven by comparison values

## 2026-07-09 - Sprint 4B Kickoff

- Sprint: Sprint 4B: Chart And Stat Component Slice
- Kickoff status: started from clean `main` after fast-forward landing Sprint 4A
- Branch: `codex/sprint-4b`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4b`
- Plan file: `docs/sprints/sprint-4b-plan.md`
- Planned verification commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-web-components`
  - `cargo test -p tessara-web-datasets`
  - `cargo test -p tessara-web-data-ops`
  - `npx playwright test`
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Revised closeout documentation requirements:
  - update `docs/playwright-permissions-scenarios.md` for visual component reader, manager, and scoped negative cases
  - record the legacy visual-analysis endpoint inventory in closeout notes
- Immediate implementation focus:
  - inventory Sprint 4A ComponentVersion table contracts and define typed visual config contracts for Bar, Line, Pie/Donut, and StatCard components

## 2026-07-09 - Sprint 4B Visual Component API/UI Checkpoint

- Completed first vertical slice:
  - widened `component_type` storage from `table` to `table`, `bar`, `line`, `pie`, `donut`, and `stat_card`
  - added typed visual config validation for Bar, Line, Pie/Donut, and StatCard components over Dataset major-line fields
  - added kind-specific published execution endpoints for `/bar`, `/line`, `/pie`, `/donut`, and `/stat-card`, including versioned endpoint variants
  - kept table execution table-only; visual kinds now receive stable wrong-kind 400 errors from table endpoints
  - implemented native server-side visual transforms for grouping, summary functions, missing-value policy, sorting, and limits
  - extended component authoring with component-kind selection and visual config controls
  - extended component viewers with native Leptos route ownership, StatCard rendering, and D3-backed Bar, Line, Pie, and Donut chart rendering from ComponentVersion view models
- Completed follow-up coverage:
  - added API integration coverage that creates, publishes, and executes Bar, Line, Pie, Donut, and StatCard components over a Dataset major line
  - added explicit-version visual execution coverage for a superseded Bar version after the current published component switches to Line
  - added wrong-kind visual endpoint coverage and `/stat_card` 404 coverage
  - added Playwright coverage for UI-driven visual component authoring and publishing for Bar, Line, Pie, Donut, and StatCard, validation, version-history review, historical visual execution, D3 visual viewer rendering, no `/bridge/*` requests, and browser-console cleanliness
  - added Playwright scoped permission coverage for visible and hidden published Bar components
  - added Sprint UAT and smoke-script visual Bar checks that create/publish/execute a visual component and load its viewer shell
- Legacy visual-analysis endpoint inventory:
  - No legacy visual-analysis endpoints touched in this checkpoint.
- Validation:
  - `cargo fmt --all` - passed
  - `cargo check -p tessara-api --lib` - passed
  - `cargo check -p tessara-web --features hydrate` - passed
  - `cargo check -p tessara-web-components --features hydrate` - passed
  - `cargo test -p tessara-api` - passed: 75 component/dataset unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `cargo test -p tessara-web` - passed: 4 web shell tests and doc tests
  - `cargo test -p tessara-web-components` - passed: 17 component UI helper tests and doc tests
  - `cargo test -p tessara-web-datasets -p tessara-web-data-ops` - passed: 24 dataset UI helper tests, 1 data-ops helper test, and doc tests
  - `cargo test -p tessara-api --test demo_flow dataset_revision_draft_publish_preserves_current_until_publish` - passed after adding visual endpoint coverage
  - `.\scripts\local-launch.ps1 -FreshData -SkipSeed` - passed after redeploying Sprint 4B on the normal local ports
  - `PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npx playwright test tests/components.spec.ts` - passed: table component workflow and visual component workflow, including UI-created/published Bar, Line, Pie, Donut, and StatCard components
  - `PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npx playwright test` - passed: 40/40 browser tests
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"` - passed: 40/40 browser tests, including permission scenarios
  - PowerShell parse checks for `scripts/smoke.ps1` and `scripts/uat-sprint.ps1` - passed
  - `.\scripts\smoke.ps1` - passed on normal local ports: 6/6 demo-flow tests plus visual Bar publish/execute smoke evidence with `visual_points = 2`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed against a fresh rebuilt Sprint 4B stack
- Status:
  - Sprint 4B implementation and closeout validation are complete on the normal local stack.

## 2026-07-10 - Sprint 4B Browser Feedback Hardening

- Completed browser-feedback refinements for visual component authoring and viewing:
  - D3-based chart rendering is wired into visual component viewers while retaining ComponentVersion as the source of truth.
  - Demo Session Log seed data now supports a richer component-testing dataset and one seeded example component per visual kind.
  - Component editor panels, field help, component-kind descriptions, category display labels, category colors, and legend title controls were refined from in-browser review.
  - Category Display settings now reset stale per-category labels and colors when the author changes the Dataset Version or Category Field, while preserving saved labels/colors when reopening a component on its saved field.
- Added regression coverage:
  - `end2end/tests/components.spec.ts` creates a Donut draft with saved `false`/`true` category labels, switches the Category Field to `Topics Covered (multi_choice)`, and verifies the Category Display table contains only the new topic groups.
  - `crates/tessara-api/tests/demo_flow.rs` executes the seeded Demo Session Log Bar, Line, Pie, Donut, and StatCard components by slug and verifies their view models carry seeded labels, colors, legend title, and StatCard supporting text.
  - `scripts/smoke.ps1` and `scripts/uat-sprint.ps1` now exercise the seeded Demo Session Log visual examples by slug and tolerate an already-seeded local database after `local-launch.ps1 -FreshData`.
- Validation:
  - `cargo check -p tessara-web-components -p tessara-api` - passed.
  - `cargo test -p tessara-api --test demo_flow demo_seed_uses_capability_scope_ownership_and_components` - passed.
  - `cargo test -p tessara-web document::assets::tests` - passed: 3/3 document asset tests, including the D3 chart renderer asset and document head script tags.
  - `PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npx playwright test tests/components.spec.ts` - passed: 2/2 component workflow tests.
  - `.\scripts\local-launch.ps1` - passed after full API image rebuild and launched healthy `postgres` and `api` containers on the normal local ports.
  - `.\scripts\local-launch.ps1 -FreshData -SkipBuild` - passed and refreshed/reseeded the local database on the normal local ports.
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed against the refreshed seeded local stack.
  - `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices` - passed against the refreshed seeded local stack.
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"` - passed: 40/40 browser tests, including component visual authoring/viewing and permission scenarios.
  - `cargo test -p tessara-web -p tessara-web-components -p tessara-web-datasets -p tessara-web-data-ops` - passed: 5 web shell tests, 17 component UI helper tests, 24 dataset UI helper tests, 1 data-ops helper test, and doc tests.
  - `cargo test -p tessara-api` - passed: 75 component/dataset unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests.

## 2026-07-09 - Sprint 4A Final Closeout

- Sprint: Sprint 4A: Dataset Catalog And Thin Table Components
- Branch: `codex/sprint-4a`
- PR: `https://github.com/ericwburden/tessara/pull/103`
- Status: complete and ready for merge review, with Sprint 4B marked next in the roadmap
- Completed:
  - pivoted from component-owned Detail/Aggregate Tables to one thin `table` component over Dataset major-line outputs
  - added Dataset catalog tags, searchable Dataset discovery, Dataset detail tabs, and editable catalog metadata that does not create Dataset revisions
  - added Dataset provenance lineage as a tree rooted at the current Dataset, with Form and Dataset ancestors linked from the lineage view
  - simplified Component storage and APIs to table-only, major-line-only versions with `dataset_id`, `dataset_version_major`, `binding_mode = major_line`, `component_type = table`, and optional version notes
  - removed the old `/components/:component_ref/publish` interstitial route and moved authoring decisions into the edit screen
  - added edit-screen version actions: `Save Draft`, `Update Existing Version`, and `Create New Version`
  - added the Create New Version consumer-review placeholder modal with searchable consumer list space and required New Version Note
  - added atomic component save behavior so component metadata, draft changes, update-existing-version, and create-new-version actions commit or fail together
  - added shared table rendering for Dataset previews and Component viewers, including search, column visibility, header sort/filter menus, reset controls, pagination, and horizontal-scroll-safe menus
  - updated Component list UX with status distinctions for `Draft`, `Published`, and `Updating`, current revision display, icon actions, filters, and shared pagination footer styling
  - hid draft component metadata and authoring controls from reader-only component list/detail/version surfaces while keeping drafts manager-visible
  - removed legacy Detail Table, Aggregate Table, revision-bound component compatibility, inline publish flag, and old component kind paths from the Sprint 4A product surface
  - clarified the remaining granular component admin endpoints as API/test setup paths; the edit screen uses the atomic save command as the authoring workflow
- Validation:
  - `.\scripts\local-launch.ps1` - passed after full API image rebuild and launched `http://localhost:8080`
  - `.\scripts\local-launch.ps1 -FreshData -SkipSeed` - passed and launched a clean empty stack for UAT seeding
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed for organization, forms, datasets, components, and seed flows
  - `.\scripts\smoke.ps1` - passed: 6 demo-flow smoke tests plus JSON smoke summary
  - `cargo fmt --all` - passed
  - `cargo test -p tessara-api` - passed: 71 component/dataset unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `cargo test -p tessara-web` - passed: 4 web shell tests and doc tests
  - `cargo test -p tessara-web-components -p tessara-web-datasets -p tessara-web-data-ops` - passed: 17 component tests, 1 data-ops test, 24 dataset tests, and doc tests
  - from `end2end/`: `npx playwright test` - passed: 39/39 browser tests
  - final `.\scripts\local-launch.ps1 -FreshData` - passed and relaunched the seeded app for handoff
- Next Sprint: Sprint 4B: Chart And Stat Component Slice

### Sprint Handoff / Demo Instructions

#### Dataset Catalog Search And Tags
- Role: admin
- Paths:
  - `http://localhost:8080/datasets`
  - `http://localhost:8080/datasets/{dataset_id}`
  - `http://localhost:8080/datasets/{dataset_id}/edit`
- Steps:
  1. Log in as `admin@tessara.local / tessara-dev-admin`.
  2. Open `Datasets` and search by Dataset name, slug, grain, tag, field label/key, and provenance source name.
  3. Open a Dataset detail page and review its tag display.
  4. Use `Edit Dataset` to add and remove tags with the combobox/chip control.
- Expected:
  - Dataset directory and detail surfaces expose tags, and search responds to tags plus source/field metadata.
- Acceptance check:
  - Pass when tag edits persist and the directory search can rediscover the Dataset by the saved tag.
- Evidence location:
  - `end2end/tests/components.spec.ts`
  - `end2end/tests/datasets.spec.ts`
  - `cargo test -p tessara-web-datasets`

#### Dataset Provenance Lineage
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}`
- Steps:
  1. Open a Dataset with contributing Forms or upstream Datasets.
  2. Review the direct `Sources` view.
  3. Open the `Provenance` view and inspect the full ancestor tree rooted at the current Dataset.
- Expected:
  - Form and Dataset ancestors are visible with distinct lineage entries and links where applicable.
- Acceptance check:
  - Pass when the Dataset detail distinguishes direct sources from full provenance lineage.
- Evidence location:
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web-datasets`
  - `npx playwright test`

#### Thin Table Component Authoring
- Role: admin
- Paths:
  - `http://localhost:8080/components`
  - `http://localhost:8080/components/new`
  - `http://localhost:8080/components/{component_ref}/edit`
- Steps:
  1. Open `Components` and create a new component.
  2. Choose a Dataset major-line source and review the Dataset Context panel.
  3. Configure Displayed Fields, Default Filters, default sort, and page size.
  4. Save as draft, update an existing published version, or create a new version with a version note.
- Expected:
  - Component authoring stores only table presentation config and binds to a Dataset major line, not a Dataset revision.
- Acceptance check:
  - Pass when a draft can be saved, a version can be published, and the old publish interstitial route is absent.
- Evidence location:
  - `end2end/tests/components.spec.ts`
  - `crates/tessara-api/tests/demo_flow.rs`
  - `cargo test -p tessara-web-components`

#### Component Viewer And Version History
- Role: admin
- Paths:
  - `http://localhost:8080/components/{component_ref}`
  - `http://localhost:8080/components/{component_ref}/view`
  - `http://localhost:8080/components/{component_ref}/versions`
  - `GET /api/components/{component_ref}/table`
  - `GET /api/components/{component_ref}/versions/{version_id}/table`
- Steps:
  1. Open a published Component from the directory.
  2. Confirm the default route renders the table preview with the shared interactive table.
  3. Use search, column controls, reset, header sort/filter menus, pagination, and horizontal scrolling.
  4. Open version history and confirm version notes appear for created versions.
- Expected:
  - Component viewers render published table data and keep superseded published-history versions readable when scoped access allows.
- Acceptance check:
  - Pass when the viewer displays rows from the bound Dataset major line and viewer controls do not mutate component config.
- Evidence location:
  - `end2end/tests/components.spec.ts`
  - `end2end/tests/permissions.spec.ts`
  - `.\scripts\smoke.ps1`

#### Reader And Scoped Governance
- Role: operator
- Paths:
  - `http://localhost:8080/components`
  - `http://localhost:8080/components/{component_ref}`
  - `http://localhost:8080/components/{component_ref}/versions`
  - `POST /api/admin/components/validate`
  - `POST /api/admin/components/{component_id}/versions/{version_id}/publish`
- Steps:
  1. Log in with a scoped non-admin account from the seeded demo set.
  2. Confirm hidden Datasets and hidden Components are absent or forbidden.
  3. Confirm reader component surfaces do not show draft metadata, create/edit actions, or draft-only versions.
  4. Attempt an out-of-scope component bind or publish through the scripted permission fixtures.
- Expected:
  - Readers see only published visible components; managers see only manageable drafts/versions; guessed out-of-scope IDs are denied.
- Acceptance check:
  - Pass when reader-only users cannot discover drafts or authoring controls, and scoped managers cannot bind/publish hidden Dataset major lines.
- Evidence location:
  - `end2end/tests/permissions.spec.ts`
  - `components::tests::reader_component_summary_omits_absent_draft_metadata`
  - `pages::tests::reader_component_summary_without_draft_metadata_stays_published`

### Acceptance Mapping

- Exit condition:
  - A tester can tag and discover Datasets, review direct provenance, then create, version, publish, and view thin table components in the app.
- Manual demonstration:
  - `Dataset Catalog Search And Tags`, `Dataset Provenance Lineage`, `Thin Table Component Authoring`, and `Component Viewer And Version History`.
- Automated check:
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`, `npx playwright test`, `cargo test -p tessara-api`, `cargo test -p tessara-web-components -p tessara-web-datasets -p tessara-web-data-ops`.

- Exit condition:
  - Dataset APIs expose tags and provenance summaries; directory search matches tags, fields, Forms, and upstream Datasets; Dataset managers can edit tags.
- Manual demonstration:
  - `Dataset Catalog Search And Tags` and `Dataset Provenance Lineage`.
- Automated check:
  - `end2end/tests/datasets.spec.ts`, `end2end/tests/components.spec.ts`, `cargo test -p tessara-web-datasets`, `cargo test -p tessara-api`.

- Exit condition:
  - Component versions bind to Dataset major version lines, are constrained to `component_type = table`, and store only presentation-level table options.
- Manual demonstration:
  - `Thin Table Component Authoring`.
- Automated check:
  - `crates/tessara-api/src/components/mod.rs` unit tests for legacy kind rejection, legacy revision payload rejection, presentation-field validation, and config validation.

- Exit condition:
  - Component validation rejects unknown fields, sorts, search fields, unsupported kinds, and invalid saved filters before reader execution.
- Manual demonstration:
  - `Thin Table Component Authoring`.
- Automated check:
  - `cargo test -p tessara-api`, `cargo test -p tessara-web-components`, and Playwright component validation assertions.

- Exit condition:
  - Component table execution renders the Dataset major-line table surface, including compatible minor/patch rows, while later major versions do not affect a v1-bound component.
- Manual demonstration:
  - `Component Viewer And Version History`.
- Automated check:
  - `crates/tessara-api/tests/demo_flow.rs`, `end2end/tests/datasets.spec.ts`, `end2end/tests/components.spec.ts`, and `.\scripts\smoke.ps1`.

- Exit condition:
  - Draft/edit flows preserve the current published version until publish; authors can update an existing version in place or create a new version with a note and consumer-review placeholder.
- Manual demonstration:
  - `Thin Table Component Authoring` and `Component Viewer And Version History`.
- Automated check:
  - `end2end/tests/components.spec.ts`, `crates/tessara-api/tests/demo_flow.rs`, and `cargo test -p tessara-api`.

- Exit condition:
  - Public reader routes and dashboards consume only published-history component versions; drafts remain admin-only authoring state.
- Manual demonstration:
  - `Reader And Scoped Governance`.
- Automated check:
  - `end2end/tests/permissions.spec.ts`, `components::tests::reader_component_summary_omits_absent_draft_metadata`, and `pages::tests::reader_component_summary_without_draft_metadata_stays_published`.

- Exit condition:
  - Scoped users cannot list hidden Dataset major lines, bind guessed hidden Dataset IDs, publish out-of-scope component versions, or direct-load hidden component detail/table routes.
- Manual demonstration:
  - `Reader And Scoped Governance`.
- Automated check:
  - `end2end/tests/permissions.spec.ts` scoped component and historical version checks plus API demo-flow scope assertions.

- Exit condition:
  - Touched component routes remain native Leptos SSR routes and the old component publish page is absent.
- Manual demonstration:
  - Open `/components`, `/components/new`, `/components/{component_ref}`, `/components/{component_ref}/edit`, `/components/{component_ref}/versions`, and `/components/{component_ref}/view`.
- Automated check:
  - `end2end/tests/components.spec.ts`, `end2end/tests/app.spec.ts`, `cargo test -p tessara-web`, and route adapter review in `crates/tessara-web/src/routes/components.rs`.

## 2026-07-05 - Sprint 4A Dataset Catalog And Thin Table Pivot

- Sprint pivot:
  - replaced the Detail Table / Aggregate Table split with one thin `table` component kind whose config owns one last-mile projection, one saved default filter set, display labels, default sort, page size, and viewer affordances
  - moved analytical shaping back to Datasets as the source of truth; grouped or aggregated displays should bind to display-ready Dataset major lines
  - added Dataset catalog tags and direct-source provenance summaries to improve Dataset discoverability as the catalog grows
- Final product shape:
  - Component authoring uses one Table flow; the former `/components/:component_ref/publish` interstitial route was removed
  - publishing happens from the edit screen, where authors choose whether to update the existing published version in place or create a new version
  - edit-screen save/publish actions use one backend command endpoint so component metadata and version changes commit or fail together
  - creating a new version opens a consumer-review modal with a version note; the note is shown in component version history
  - draft-only components remain visible to managers, show `Draft` status, and can be edited/published from the edit screen
  - components with a current published version plus a pending draft show `Updating`
  - component version storage is table-only and major-line-only: `dataset_id`, `dataset_version_major`, `binding_mode = major_line`, and `component_type = table`
- Completed:
  - rewrote `docs/sprints/sprint-4a-plan.md` around "Thin Table Components + Dataset provenance/search"
  - added `dataset_tags`, tag update API, tag normalization, Dataset list/detail tag loading, and direct provenance summaries
  - updated Dataset directory/detail UI so search includes tags, provenance names, field labels/keys, grain, and slug; detail shows tags and provenance tabs
  - moved Dataset tag editing into Dataset authoring with combobox/chip editing and custom tag creation
  - replaced direct provenance tables with a full ancestor lineage tree that distinguishes Forms and Datasets
  - simplified backend component validation/execution to a single `table` config with `visible_columns`, saved `filters`, `search_fields`, `default_sort`, `page_size`, and display-label overrides
  - simplified component authoring UI to one Table flow with Dataset catalog context, displayed fields, default filters, default sort, and page size
  - added pre-publish and runtime validation for saved/default filter literals so invalid numeric/date/boolean filters fail with controlled validation errors before reader execution
  - introduced shared projection/filter/collapsible-panel controls in neutral UI/data-ops boundaries instead of duplicating Dataset authoring controls inside Components
  - introduced the shared interactive table display for Dataset previews and Component rendering, with search, column visibility, header sort/filter menus, pagination, and reset controls
  - rewrote component Playwright coverage around thin table behavior instead of component-owned aggregation
  - updated roadmap and permission-scenario docs to remove stale aggregate/detail component language
  - removed the old component publish page and updated validation/UAT scripts to exercise the edit-screen publish workflow
  - squashed Sprint 4A schema work into the baseline migration because the development database resets between sprints
- Validation so far:
  - `cargo fmt --all --check` - passed
  - `cargo test -p tessara-api` - passed: 59 unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `cargo test -p tessara-web` - passed: 4 tests and doc tests
  - `cargo test -p tessara-web-components` - passed: 14 tests and doc tests
  - `cargo test -p tessara-web-datasets` - passed: 27 tests and doc tests
  - `.\scripts\smoke.ps1` - passed
  - `.\scripts\local-launch.ps1 -FreshData -SkipSeed; .\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed for organization, forms, datasets, components, and seed flows
  - from `end2end/`: `npx playwright test tests/components.spec.ts tests/permissions.spec.ts` - passed: 21/21
  - from `end2end/`: `npx playwright test` - passed: 39/39
  - post-cleanup checks passed: `cargo fmt --all --check`, `cargo check -p tessara-api`, focused Playwright component spec discovery, and `cargo clippy -p tessara-data-ops -p tessara-web-components -p tessara-web -p tessara-api --features tessara-web-components/hydrate,tessara-web/hydrate -- -D warnings`
- Current local handoff state:
  - the Tessara stack is running at `http://localhost:8080`
  - UAT seeded the refreshed database during validation, and the local database has the Sprint 4A table-only component constraint applied

## 2026-07-03 - Sprint 4A Kickoff

- Sprint: Sprint 4A: Table Component Slice
- Kickoff status: started from clean `main` with roadmap `(Next)` scope
- Branch: `codex/sprint-4a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4a`
- Plan file: `docs/sprints/sprint-4a-plan.md`
- Planned verification commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-web-components` if the new crate exposes runnable tests
  - `npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate implementation focus: inventory existing component/reporting contracts, define table component API/versioning/validation behavior, then create the `tessara-web-components` crate and root route adapters for component directory/detail/create/edit/publish/viewer flows.

## 2026-07-03 - Sprint 3C Final Closeout Validation

- Completed:
  - reran Sprint 3C on a clean local deployment and reconciled validation drift found by the full closeout suite
  - aligned dataset revision tests with the current `Version N` append-all materialization model and scoped component/dashboard visibility
  - kept publish response display semantics explicit with `semantic_version` returning `vM.m.p`
  - updated smoke validation to accept append-all major-line rows and aliased dataset output keys
  - updated Playwright permission setup so the full browser suite can reuse an already seeded clean deployment instead of failing the empty-database seed guard
  - refreshed workflow integration fixtures to match current workflow create payloads using `available_node_ids`
- Validation:
  - `cargo fmt --all` and `git diff --check` - passed
  - `cargo test -p tessara-api --test demo_flow` - passed: 6/6 tests
  - `.\scripts\smoke.ps1` - passed
  - `cargo test -p tessara-api` - passed: 41 unit tests, 6 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `cargo test -p tessara-web` - passed: 4 web tests and doc tests
  - `.\scripts\local-launch.ps1 -FreshData` - passed and launched a seeded local stack at `http://localhost:8080`
  - `npx playwright test` from `end2end/` - passed: 36/36 browser tests
  - `.\scripts\local-launch.ps1 -FreshData -SkipSeed; .\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed for organization, forms, datasets, and seed flows against an empty refreshed database
- Current local handoff state:
  - the Tessara stack is running at `http://localhost:8080`
  - demo accounts remain `admin@tessara.local`, `operator@tessara.local`, `delegator@tessara.local`, `respondent@tessara.local`, and `delegate@tessara.local` with their standard `tessara-dev-*` passwords
  - the current database was refreshed during UAT and seeded by the UAT script
- Next Sprint:
  - Sprint 4A: Table Component Slice
- Future Work:
  - Add a full workflow publish-scope review for branching and sibling step form scopes. The current workflow publisher allows combinations that older workflow-runtime expectations treated as publish-invalid; that appears unrelated to Sprint 3C dataset work, but it needs a product decision and follow-up tests so the behavior is intentional rather than accidental.

### Sprint Handoff / Demo Instructions

#### Dataset Major-Line Revision Flow
- Role: admin
- Paths:
  - `http://localhost:8080/datasets`
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `http://localhost:8080/datasets/{dataset_id}/revisions`
  - `http://localhost:8080/datasets/{dataset_id}/revisions/{revision_id}`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open a seeded dataset and review the current `vM.m.p` revision history.
  3. Edit the dataset, save a draft revision, add a revision label/notes, and review the Changelog.
  4. Publish normally or choose the publish menu's new-major path.
  5. Use another dataset's source picker to choose an upstream `Version N`.
- Expected:
  - revision detail shows semantic version, optional label, notes, changelog, dependency review, output fields, and generated SQL
  - `Version N` means the prebuilt append-all major-line table for that major version
  - compatible minor/patch publishes update consumers bound to that major line; new major publishes leave existing `Version N` consumers unchanged
- Acceptance check:
  - pass when exact revisions remain pinned, major-line sources compile from `dataset_major_materializations`, and source picker fields stay on the selected major line after a newer major exists
- Evidence location:
  - `crates/tessara-api/tests/demo_flow.rs` -> `dataset_revision_draft_publish_preserves_current_until_publish`
  - `end2end/tests/datasets.spec.ts` -> `dataset source picker keeps Version N major-line fields after a newer major exists`

#### Closeout Validation Reproduction
- Role: developer
- Commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1 -FreshData`
  - from `end2end/`: `npx playwright test`
  - `.\scripts\local-launch.ps1 -FreshData -SkipSeed; .\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Expected:
  - all commands pass
  - UAT seed path starts from an empty refreshed database
  - the app remains available at `http://localhost:8080` for manual review

### Acceptance Mapping

- Semantic revision publishing:
  - covered by API demo-flow tests for draft save, no-op publish guard, normal publish, new-major publish, semantic version response, exact-revision pinning, and `Version N` major-line retention
- Major-line materialization:
  - covered by API assertions that same-major publishes rebuild append-all tables, older rows NULL-fill fields added later, and major-line consumers remain previewable/editable/publishable after upstream moves to a newer major
- Changelog and publish guard:
  - covered by dataset unit tests for major/minor/patch version-impact classification and the no-empty-changelog publish guard
- Scoped dependency visibility:
  - covered by API and Playwright permission tests for scoped revision history/detail behavior and scoped dependency summary visibility
- Native dataset UI:
  - covered by Playwright dataset revision history/detail/publish paths, source picker `Version N` fields, repeated navigation, SQL preview, and operation state reload coverage

## 2026-06-28 - Sprint 3C Dataset Revision And Compatibility Closeout

- Completed:
  - delivered typed dataset revision, compatibility, dependency, and carry-forward contracts across API and web DTOs
  - added draft revision save, revision history/detail, and publish APIs
  - kept draft saves isolated from current published dataset catalog rows and materialized output until publish
  - added revision history/detail UI under `/datasets/{dataset_id}/revisions`
  - changed existing dataset edits to save as draft revisions and redirect to draft review before publish
  - surfaced compatibility findings and downstream dependency impact for dependent datasets, component versions, and dashboards
  - proved dependent datasets remain pinned to their referenced revision after publish
  - added read-only scoped user and scoped manager revision permission coverage
  - fixed native SSR shell routes for direct revision history/detail navigation
- Validation:
  - `cargo fmt --all` - passed
  - `cargo check -p tessara-api` - passed
  - `cargo check -p tessara-web --features hydrate` - passed
  - `cargo check -p tessara-web --no-default-features --features ssr` - passed
  - `cargo test -p tessara-api` - passed: 32 dataset unit tests, 5 demo-flow tests, 25 workflow-runtime tests, and doc tests
  - `$env:RUSTFLAGS='-C debuginfo=0'; cargo test -p tessara-web` - passed 4 web tests
  - `cargo test -p tessara-web-datasets --features hydrate --lib` - passed 30 tests
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:18080"` - passed for organization, forms, datasets, and seed flows
  - `$env:COMPOSE_PROJECT_NAME='tessara-sprint-3c-e2e'; .\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:18080" -PlaywrightArgs @("--workers=1")` - passed 33/33 browser tests, including permission scenarios
  - `git diff --check` - passed
  - Default-port `.\scripts\local-launch.ps1`, `.\scripts\smoke.ps1`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` were not used for the closeout run because `8080` and `5432` are currently owned by a neighboring `tessara-refactoring` compose stack. The Sprint 3C app was left running in an isolated compose project at `http://127.0.0.1:18080` with Postgres mapped to `15432`.
- Next Sprint:
  - Sprint 4A: Table Component Slice

### Sprint Handoff / Demo Instructions

#### Draft Revision Review And Publish
- Role: admin
- Paths:
  - `http://127.0.0.1:18080/datasets`
  - `http://127.0.0.1:18080/datasets/{dataset_id}/edit`
  - `http://127.0.0.1:18080/datasets/{dataset_id}/revisions/{revision_id}`
  - `POST /api/admin/datasets/{dataset_id}/draft-revision`
  - `POST /api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open an existing dataset, choose `Edit Dataset`, change the definition, and choose `Save Dataset`.
  3. Confirm the browser lands on a draft revision review route.
  4. Review the status, compatibility summary, downstream dependencies, output fields, and generated SQL.
  5. Choose `Publish Revision`.
- Expected:
  - saving an edit creates a draft review page instead of replacing the current published dataset immediately
  - publishing marks the draft as published current and supersedes the prior published revision
- Acceptance check:
  - pass when current dataset detail/table stay unchanged before publish and then reflect the draft after publish
- Evidence location:
  - `end2end/tests/datasets.spec.ts` -> `admin can review and publish a dataset draft revision`
  - `crates/tessara-api/tests/demo_flow.rs` -> `dataset_revision_draft_publish_preserves_current_until_publish`

#### Revision History And Status Visibility
- Role: admin
- Paths:
  - `http://127.0.0.1:18080/datasets/{dataset_id}`
  - `http://127.0.0.1:18080/datasets/{dataset_id}/revisions`
  - `GET /api/datasets/{dataset_id}/revisions`
- Steps:
  1. Open a dataset detail page.
  2. Choose `Revision History`.
  3. Confirm the list shows version labels, status, current marker behavior, field count, compatibility summary, dependency counts, and published metadata.
  4. Open a published revision and a draft revision when present.
- Expected:
  - current published, superseded, and draft revisions are distinguishable with typed statuses
  - direct browser navigation to revision history/detail serves the native Leptos shell
- Acceptance check:
  - pass when one revision is current after publish, the previous published revision is superseded, and draft rows appear only before publish
- Evidence location:
  - `end2end/tests/datasets.spec.ts` -> `admin can review and publish a dataset draft revision`
  - `crates/tessara-api/src/lib.rs` native shell routes for `/datasets/{dataset_id}/revisions`

#### Compatibility And Downstream Impact
- Role: admin
- Paths:
  - `http://127.0.0.1:18080/datasets/{dataset_id}/revisions/{revision_id}`
  - `GET /api/datasets/{dataset_id}/revisions/{revision_id}`
- Steps:
  1. Create or select a dataset that has a dependent dataset, component version, or dashboard reference.
  2. Save a draft that adds or changes an output field.
  3. Open the draft revision detail.
  4. Review `Changelog` and `Downstream Dependencies`.
  5. Publish and verify dependent assets remain pinned to their original revision.
- Expected:
  - changelog entries classify added, removed, type-changed, label, restriction-policy, and source-pipeline changes by `major`, `minor`, or `patch` version impact
  - dependency impact lists datasets, component versions, and dashboards without automatically repointing them
- Acceptance check:
  - pass when dependency impact is visible before publish and downstream dataset source bindings still reference the original revision after publish
- Evidence location:
  - `crates/tessara-api/tests/demo_flow.rs` -> dependent dataset pinning and seeded component/dashboard dependency assertions
  - `end2end/tests/datasets.spec.ts` -> downstream dependency messaging in draft review

#### Scoped Revision Permissions
- Role: admin for fixture setup; scoped dataset manager and read-only operator for verification
- Paths:
  - `GET /api/datasets/{dataset_id}/revisions`
  - `GET /api/datasets/{dataset_id}/revisions/{revision_id}`
  - `POST /api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish`
- Steps:
  1. Use admin APIs to create a scoped `datasets:manage` account assigned to the dataset visibility node.
  2. Create a second scoped manager assigned only to a child activity outside the dataset's full visibility boundary.
  3. Save a draft revision.
  4. Read history/detail and publish as the in-scope manager.
  5. Attempt draft read and publish as the out-of-scope manager and a read-only operator.
- Expected:
  - in-scope managers can see and publish drafts for datasets fully inside their scope
  - read-only users and out-of-scope managers cannot read or publish draft revisions
  - read-only revision history omits draft revisions
- Acceptance check:
  - pass when draft read/publish returns forbidden for users without matching scoped manage access, while published revisions remain readable to permitted readers
- Evidence location:
  - `crates/tessara-api/tests/demo_flow.rs` -> scoped manager and read-only operator assertions
  - `docs/playwright-permissions-scenarios.md` -> Dataset revisions scenario row

### Acceptance Mapping

- Exit condition: a tester can edit an existing dataset, save a draft revision, review compatibility findings and downstream dependencies, then publish the revision.
  - Manual demonstration: Draft Revision Review And Publish; Compatibility And Downstream Impact.
  - Automated check: `end2end/tests/datasets.spec.ts` `admin can review and publish a dataset draft revision`; `cargo test -p tessara-api --test demo_flow dataset_revision_draft_publish_preserves_current_until_publish`.
- Exit condition: published revision history clearly shows current, superseded, and draft revisions with typed statuses.
  - Manual demonstration: Revision History And Status Visibility.
  - Automated check: `crates/tessara-api/tests/demo_flow.rs` `assert_revision_statuses`; Playwright revision history status assertions.
- Exit condition: downstream components, dashboards, and dependent datasets remain pinned to their referenced revision after publish, and the UI explains downstream impact.
  - Manual demonstration: Compatibility And Downstream Impact.
  - Automated check: `crates/tessara-api/tests/demo_flow.rs` dependent dataset pinning and seeded component/dashboard dependency assertions; Playwright downstream dependency text assertion.
- Exit condition: compatibility and dependency contracts are typed in API and web DTOs; implementation does not rely on scattered raw string comparisons.
  - Manual demonstration: Revision detail summary and findings tables show typed labels from API/web contracts.
  - Automated check: `cargo test -p tessara-api datasets:: --lib`; `cargo test -p tessara-web-datasets --features hydrate --lib`; DTO definitions in `crates/tessara-api/src/datasets/dto.rs` and `crates/tessara-web-datasets/src/types/contracts.rs`.
- Exit condition: existing dataset preview, create, edit, source catalog, restriction, and materialization behavior continue to work.
  - Manual demonstration: Draft Revision Review And Publish plus the existing dataset authoring editor flow.
  - Automated check: `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:18080" -PlaywrightArgs @("--workers=1")`; `cargo test -p tessara-api`; `cargo test -p tessara-web-datasets --features hydrate --lib`.
- Exit condition: scoped dataset reader/manager accounts can only see or publish revisions permitted by their dataset capabilities.
  - Manual demonstration: Scoped Revision Permissions.
  - Automated check: `crates/tessara-api/tests/demo_flow.rs` scoped manager/read-only operator assertions; `docs/playwright-permissions-scenarios.md` Dataset revisions matrix entry.

## 2026-06-28 - Sprint 3C Dataset Revision And Compatibility Kickoff

- Kickoff status: started from clean `main` after committing accepted Sprint 3C planning artifacts.
- Sprint: Sprint 3C: Dataset Revision And Compatibility Slice.
- Branch: `codex/sprint-3c`.
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-3c`.
- Plan file: `docs/sprints/sprint-3c-plan.md`.
- Planned verification commands: `cargo fmt --all`, `cargo test -p tessara-api`, `cargo test -p tessara-web`, `npx playwright test`, `.\scripts\smoke.ps1`, `.\scripts\local-launch.ps1`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.
- Immediate implementation focus: typed revision, compatibility, dependency, and carry-forward contracts, followed by revision list/detail/draft/publish API behavior.

## 2026-06-28 - Sprint 3C Revision Lifecycle Checkpoint

- Implemented backend revision lifecycle foundation: typed revision/compatibility/dependency/carry-forward DTOs, `definition_metadata` revision snapshots, one-open-draft baseline constraint, draft-save endpoint, revision history/detail endpoints, and draft publish endpoint.
- Draft saves now compile and store revision snapshots without mutating current-published dataset catalog tables or materialized preview state; publish atomically updates dataset metadata, visibility, current source/field catalogs, revision status, and materialized output.
- Added initial compatibility classification for added, removed, type-changed, label-changed, restriction-policy-changed, and source-pipeline-changed revision snapshots, plus dependency impact discovery through dependent datasets, component versions, and dashboards.
- Implemented first web slice: existing dataset save posts to draft-save and redirects to draft review; dataset detail links to revision history; revision history and revision detail/review routes are available under `/datasets/:dataset_id/revisions`.
- Added database-backed API coverage proving draft saves do not mutate current published detail/table state, revision history reports draft/published/superseded states, publish advances the current revision, dependent datasets remain pinned, component/dashboard dependencies surface from seeded data, and read-only scoped users cannot view or publish draft revisions.
- Verification passed: `cargo fmt --all`, `cargo check -p tessara-api`, `cargo check -p tessara-web --features hydrate`, `cargo check -p tessara-web --no-default-features --features ssr`, `cargo test -p tessara-api datasets:: --lib`, `cargo test -p tessara-api --test demo_flow`, and `cargo test -p tessara-web-datasets --features hydrate --lib`.
- Remaining focus: polish revision UI, add Playwright coverage, cover additional scoped manager edge cases, and run the full sprint validation set.

## 2026-06-28 - Sprint 3C Revision Lifecycle E2E Checkpoint

- Added Playwright coverage for the admin draft review and publish flow: draft save leaves the current published dataset unchanged, revision history shows published/draft states, draft detail surfaces compatibility findings and downstream dependencies, publish advances the current revision, and dependent datasets remain pinned to their original revision.
- Updated legacy Sprint 3A/3B dataset authoring Playwright flows to follow the new draft-save contract: UI saves redirect to revision review, tests publish the draft before asserting current dataset detail, and direct navigation returns to the main dataset detail where needed.
- Fixed native shell route registration for `/datasets/{dataset_id}/revisions` and `/datasets/{dataset_id}/revisions/{revision_id}` so direct SSR navigation and redirected draft-review URLs serve the Leptos app instead of returning 404.
- Local e2e stack note: validated with an isolated compose project on `http://127.0.0.1:18080` because an existing `tessara-refactoring` stack owned `8080` and `5432`.
- Verification passed: `cargo fmt --all`, `cargo check -p tessara-api`, `cargo check -p tessara-web --features hydrate`, `cargo test -p tessara-api datasets:: --lib`, `cargo test -p tessara-api --test demo_flow`, `npm --prefix end2end exec -- playwright test --list tests/datasets.spec.ts`, focused `npm --prefix end2end test -- tests/datasets.spec.ts -g "admin can review and publish a dataset draft revision"`, and full `npm --prefix end2end test -- tests/datasets.spec.ts`.
- Remaining focus: additional scoped manager edge cases, final revision UI polish, and the full sprint closeout validation set.

## 2026-06-28 - Sprint 3C Scoped Manager Revision Checkpoint

- Added API regression coverage for scoped dataset managers in the revision lifecycle test: an in-scope `datasets:manage` user can read draft revision detail, sees draft rows in revision history, and can publish the draft; a manager scoped only to a child activity is forbidden from reading or publishing the parent-program dataset draft.
- Polished revision history heading copy so the app-level title remains `Dataset Revisions` while the inner page header reads `Revision History`.
- Verification passed: `cargo fmt --all`, focused `cargo test -p tessara-api --test demo_flow dataset_revision_draft_publish_preserves_current_until_publish`, full `cargo test -p tessara-api --test demo_flow`, `cargo check -p tessara-web --features hydrate`, `cargo check -p tessara-web --no-default-features --features ssr`, and `git diff --check`.
- Remaining focus: full sprint closeout validation.

## 2026-06-28 - Sprint 3C Validation Checkpoint

- Rebuilt the isolated Sprint 3C local stack on `http://127.0.0.1:18080` / Postgres `15432` after the final route and UI changes. The default `8080` / `5432` ports were occupied by a separate `tessara-refactoring` stack.
- Validation passed: `cargo fmt --all`, `cargo check -p tessara-api`, `cargo check -p tessara-web --features hydrate`, `cargo check -p tessara-web --no-default-features --features ssr`, `cargo test -p tessara-api datasets:: --lib`, `cargo test -p tessara-api --test demo_flow`, `cargo test -p tessara-web-datasets --features hydrate --lib`, `npm --prefix end2end test -- tests/datasets.spec.ts` with `PLAYWRIGHT_BASE_URL=http://127.0.0.1:18080`, `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:18080"`, and `git diff --check`.
- Not run in this worktree: `.\scripts\smoke.ps1` and `.\scripts\local-launch.ps1` against their default `8080` / `5432` ports, because those ports are currently owned by the neighboring `tessara-refactoring` compose stack.
- Remaining focus: sprint closeout packaging and handoff notes.

## 2026-06-26 - Sprint 3C Dataset Authoring Refactor Checkpoint

- Unified the dataset editor source-composition UI around one Add Source operation draft with an Add Type selector for Union, Union All, Left Join, Inner Join, and Outer Join.
- Unified backend and web source-composition contracts around `add_source`, with `add_type`, source, join keys, and position carrying Union, Union All, Left Join, Inner Join, and Outer Join behavior.
- Removed the legacy source-composition operation variants from backend DTOs, web payload contracts, editor loading, API fixtures, and Playwright request fixtures so active callers use `add_source` directly.
- Sprint closeout gate: run full validation and confirm stored revision compatibility/migration against the unified `add_source` contract before Sprint 3C can close.

## 2026-06-19 - Sprint 3B Dataset Advanced Authoring Closeout

- Completed:
  - delivered advanced dataset authoring while preserving the Sprint 3A editor flow: Definition, Sources, Fields, Aggregation, Calculated Fields, Filters, SQL Preview, Visibility
  - added output-level row filters with type-aware operators, literal value and field-comparison modes, typed value controls, persistence, hydration, SQL preview, and materialized-row behavior
  - added calculated fields as first-class output columns with ordered function pipelines, carry-forward type-aware function options, typed literal and field arguments, expression previews, comparison/casting/map/default functions, and save/reopen hydration
  - added hidden included fields through the `Display?` model so fields can feed calculations, filters, joins, aggregation, and restriction flags without appearing in normal dataset output
  - replaced the original tier-field restriction model with boolean internal/restricted/confidential flag fields, public-by-default rows, lowest-tier-wins precedence, standard restricted/confidential capabilities, and row gating in dataset table loading
  - audited and corrected SQL generation order so source operations run before projection, followed by aggregation, calculated fields, filters, and restriction-tier enforcement metadata
  - hardened validation for missing references, invalid function ids, invalid typed literals, incompatible field arguments, duplicate output keys, unsupported operators, no-visible-fields saves, and legacy flat-source payloads
  - refined the editor UAT surface with collapsible panels, responsive expression/source layouts, mobile card layouts, icon-based row actions, blur/change-based reactive updates, and clearer advanced-authoring controls
  - captured Sprint 3B reviewer demo steps directly in `end2end/tests/datasets.spec.ts` under the `admin can UAT Sprint 3B advanced dataset authoring` Playwright test
- Validation:
  - `.\scripts\local-launch.ps1` - passed; final relaunch left the app running at `http://localhost:8080` and a health check returned HTTP 200
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed for organization, forms, datasets, and seed flows
  - `.\scripts\smoke.ps1` - passed; demo-flow smoke returned `dataset_rows: 2`
  - `cargo fmt --all` - passed
  - `cargo test -p tessara-api` - passed: 15 dataset unit tests, 4 demo-flow tests, and 25 workflow-runtime tests
  - `cargo test -p tessara-web` - bare Windows MSVC run hit linker `LNK1318` PDB limit; rerun with `$env:RUSTFLAGS='-C debuginfo=0'; cargo test -p tessara-web` passed 7 tests
  - `cargo check -p tessara-web --no-default-features --features ssr` - passed
  - `cargo check -p tessara-web --features hydrate` - passed
  - `npx playwright test` - passed: 29/29 browser tests, including Sprint 3A regression authoring, Sprint 3B UAT, and constrained non-admin permission checks
- Next Sprint:
  - Sprint 3C: Dataset Revision And Compatibility Slice

### Sprint Handoff / Demo Instructions

#### Advanced Dataset Authoring Editor
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `GET /api/datasets/{dataset_id}`
  - `PUT /api/admin/datasets/{dataset_id}`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open a dataset edit page and confirm Aggregation, Calculated Fields, Filters, and View Restrictions start collapsed.
  3. Add a second source with `Add Input`, set an alias, pick the demo form/version, and confirm the first source's projected field count is unchanged.
  4. Convert the source operation from `UNION` to `INNER JOIN` and choose pre-projection node-id join keys.
  5. Include source fields needed for downstream logic, then hide helper fields with `Display?` while leaving them available in advanced dropdowns.
  6. Add calculated fields using a date comparison field argument, a numeric comparison followed by `Cast to Text`, and a text `Map Value` pipeline.
  7. Add one literal numeric filter and one date filter that compares against another field.
  8. Save, reopen, and verify calculations, filters, hidden fields, and source selections hydrate without clearing.
- Expected:
  - field options include all included fields from all sources, hidden fields are subtly labeled, filters and calculations update on blur/change, and generated SQL/save state reflect the authored controls
- Acceptance check:
  - pass when the saved detail payload includes `row_filters`, `calculated_fields`, hidden included fields, and the same authoring state after reopening
- Evidence location:
  - `end2end/tests/datasets.spec.ts` -> `admin can UAT Sprint 3B advanced dataset authoring`
  - `crates/tessara-api/tests/demo_flow.rs` -> `dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence`

#### SQL Generation And Materialized Output
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `POST /api/admin/datasets/sql-preview`
  - `GET /api/datasets/{dataset_id}/table`
- Steps:
  1. Open Generated SQL after configuring joins, hidden fields, aggregation, calculations, filters, and restrictions.
  2. Verify the SQL has source CTEs before `selected_fields`, aggregates before `calculated_fields`, `filtered_fields` after calculations, and restriction-tier metadata kept internal.
  3. Save the dataset and open the detail/table preview.
- Expected:
  - materialized output includes the final analytical field catalog, keeps restriction metadata internal, and uses typed SQL casts for numeric/date comparisons
- Acceptance check:
  - pass when SQL preview and saved materialization use the same operation order and output contract
- Evidence location:
  - `crates/tessara-api/tests/demo_flow.rs` -> `admin_dataset_query_designer_materializes_generated_sql`
  - `crates/tessara-api/src/datasets/mod.rs` unit tests for calculated-field SQL order and typed expressions

#### View Restriction Enforcement
- Role: admin for authoring; constrained non-admin operator for verification
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `GET /api/datasets/{dataset_id}/table`
  - role capability UI for `datasets:read_restricted` and `datasets:read_confidential`
- Steps:
  1. In View Restrictions, enable Internal, Restricted, and/or Confidential rows and select boolean flag fields.
  2. Save the dataset.
  3. Read the dataset table as admin and then as an operator account without confidential access.
  4. Compare visible rows for public/internal/restricted/confidential tier behavior.
- Expected:
  - public is the default row tier, enabled boolean flags set row tiers, internal wins when multiple selected flags are true, restricted rows require restricted or confidential read capability, confidential rows require confidential read capability, and `admin:all` bypasses tier filtering
- Acceptance check:
  - pass when constrained non-admin table reads omit rows above their capabilities while admin sees all permitted rows for the visible dataset
- Evidence location:
  - `crates/tessara-api/tests/demo_flow.rs` -> `dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence`
  - `crates/tessara-api/tests/demo_flow.rs` -> `admin_dataset_query_designer_materializes_generated_sql`

#### Editor Responsiveness And Mobile Handoff
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
- Steps:
  1. Resize the editor to desktop, medium, and mobile widths.
  2. Inspect the source expression builder, source field cards, aggregation metrics, calculated-field accordions, filters, restrictions, and generated SQL panel.
  3. Confirm text inputs commit on blur/change rather than keystroke and icon actions remain visible without horizontal overflow.
- Expected:
  - advanced panels use consistent collapsible headers, mobile layouts switch to cards, expression cards remain readable, and destructive/actions use compact icon buttons
- Acceptance check:
  - pass when the editor remains usable at mobile width and does not lose focus while typing advanced values
- Evidence location:
  - `end2end/tests/datasets.spec.ts` -> `admin can UAT Sprint 3B advanced dataset authoring`
  - reviewer browser-session feedback captured during Sprint 3B implementation

### Acceptance Mapping

- Exit condition: a tester can add a row filter to a dataset, preview the resulting rows, save the definition, and reopen it.
  - Manual demonstration: Advanced Dataset Authoring Editor steps 7 and 8.
  - Automated check: `end2end/tests/datasets.spec.ts` `admin can UAT Sprint 3B advanced dataset authoring`; `crates/tessara-api/tests/demo_flow.rs` `dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence`.
- Exit condition: a tester can add a calculated field to a dataset, preview SQL/materialized output, save the definition, and reopen it.
  - Manual demonstration: Advanced Dataset Authoring Editor steps 6 and 8; SQL Generation And Materialized Output steps 1-3.
  - Automated check: `end2end/tests/datasets.spec.ts` `admin can UAT Sprint 3B advanced dataset authoring`; `cargo test -p tessara-api`.
- Exit condition: explicit restriction rules behave as authored rather than being implied by system metadata.
  - Manual demonstration: View Restriction Enforcement steps 1-4.
  - Automated check: `crates/tessara-api/tests/demo_flow.rs` `dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence`; `admin_dataset_query_designer_materializes_generated_sql`.
- Exit condition: generated SQL applies operations in screen order and keeps helper columns internal.
  - Manual demonstration: SQL Generation And Materialized Output steps 1-3.
  - Automated check: `crates/tessara-api/tests/demo_flow.rs` `admin_dataset_query_designer_materializes_generated_sql`; `crates/tessara-api/src/datasets/mod.rs` SQL-order unit tests.
- Exit condition: invalid filters, missing field references, unsupported function ids, invalid argument types, and no-visible-field saves produce actionable validation errors.
  - Manual demonstration: Advanced Dataset Authoring Editor steps 5-7 with invalid values or hidden-only output.
  - Automated check: `cargo test -p tessara-api`, including typed literal, invalid function, incompatible field argument, legacy source, and no-visible-fields validation assertions.
- Exit condition: the Sprint 3A dataset authoring flow remains intact while advanced authoring is added.
  - Manual demonstration: Advanced Dataset Authoring Editor steps 1-3 and Editor Responsiveness And Mobile Handoff steps 1-3.
  - Automated check: `end2end/tests/datasets.spec.ts` `admin can author, edit, save, and view a Sprint 3A dataset` plus `admin can UAT Sprint 3B advanced dataset authoring`.

## 2026-06-17 - Sprint 3B Dataset Advanced Authoring Kickoff

- Kickoff status: started from clean `main` and created sprint branch `codex/sprint-3b`.
- Sprint worktree: `C:\Users\eric-dev\Projects\tessara-sprint-3b`.
- Plan file: `C:\Users\eric-dev\Projects\tessara-sprint-3b\docs\sprints\sprint-3b-plan.md`.
- Planned verification: `cargo fmt --all`, `cargo test -p tessara-api`, `cargo test -p tessara-web`, `npx playwright test`, `.\scripts\smoke.ps1`, `.\scripts\local-launch.ps1`, `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.
- Immediate focus: review the Sprint 3A dataset definition, SQL preview, editor state, and persistence contracts before implementing row filters, calculated fields, and explicit restriction rules.

## 2026-06-17 - Sprint 3A Dataset Authoring Foundation Closeout

- Completed:
  - shipped native Dataset directory, detail, create, edit, and preview flows at `/datasets`, `/datasets/new`, `/datasets/{dataset_id}`, and `/datasets/{dataset_id}/edit`
  - replaced placeholder dataset screens with a native authoring flow: Dataset Definition, Data Sources, Fields, Aggregation, Filters placeholder, Generated SQL, and Visibility
  - added a Data Sources expression designer for form and dataset inputs, nested expressions, union/union all, left/inner/outer joins, pre-projection join keys, and right-side configuration sheets
  - added field projection with source-grouped accordions, stable include/exclude behavior, editable display labels, system metadata fields, and stable selected-field readback after save/reopen
  - added Aggregation v1 with `None`, `Row`, and `Field` modes, grouping comboboxes, multi-field row sorting with shared direction, metric rows, and generated SQL recompilation
  - kept Filters as a read-only placeholder for Sprint 3B so the editor matches the intended flow without expanding Sprint 3A scope
  - updated generated dataset SQL to use stable logical form field identity through `(form_version_id, field_id)`, removed source-level latest/earliest/all selection, removed ranked source CTEs, and stopped treating mutable `field_key` as identity
  - simplified dataset visibility so visibility selections are the dataset read gate; materialized dataset rows are not implicitly filtered by `__node_id`
  - retained source node metadata as normal selectable fields for joins, grouping, debugging, and future explicit restriction rules
  - tightened `/api/form-versions/{id}/render` and propagated stable form field identity through create/copy/update/delete, submissions, analytics facts, dataset definitions, and tests
  - added Sprint 3A Playwright coverage in `end2end/tests/datasets.spec.ts` for dataset directory/detail/editor/viewing flows, SQL preview, aggregation, visibility, and source configuration behavior
  - confirmed the migration set is squashed for closeout: `crates/tessara-api/migrations` contains a single `001_baseline.sql`
- Validation:
  - `cargo fmt --all` - passed
  - `cargo check -p tessara-api` - passed
  - `cargo check -p tessara-web --features hydrate` - passed
  - `cargo check -p tessara-web --no-default-features --features ssr` - passed
  - `cargo test -p tessara-api` - passed
  - `cargo test -p tessara-web` - passed
  - `.\scripts\smoke.ps1` - passed after fixing the built-in operator `datasets:read` seed and updating aggregation min/max test expectations
  - `.\scripts\local-launch.ps1 -FreshData` - passed and left the local app running at `http://localhost:8080`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed
  - `npx playwright test` from `end2end` - passed, 28 tests
- Known release conditions:
  - editable dataset filters are intentionally deferred to Sprint 3B; Sprint 3A only displays the placeholder in the final editor sequence
  - explicit dataset row restriction rules are future work; Sprint 3A makes dataset visibility the read gate and keeps `__node_id` available as normal metadata
- Next Sprint:
  - Sprint 3B: Dataset Advanced Authoring Slice

### Sprint Handoff / Demo Instructions

#### Dataset Directory And Detail
- Role: admin
- Paths:
  - `http://localhost:8080/datasets`
  - `http://localhost:8080/datasets/{dataset_id}`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/datasets`.
  3. Search for a seeded dataset, open it, and switch through Preview, Sources, Fields, and SQL tabs.
  4. Click the Visibility summary card to inspect visible nodes.
- Expected:
  - the directory uses standard searchable, paginated table behavior; dataset names link to detail; slugs render as secondary text
  - detail tabs show the materialized output, sources, final output fields, and generated SQL without exposing internal helper columns
- Acceptance check:
  - pass when a reviewer can inspect a dataset and verify that Preview reflects the final materialized table output
- Evidence location:
  - `end2end/tests/datasets.spec.ts`
  - closeout Playwright transcript

#### Dataset Authoring Editor
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/new`
  - `http://localhost:8080/datasets/{dataset_id}/edit`
- Steps:
  1. Create or edit a dataset.
  2. Configure Dataset Definition metadata.
  3. Use Data Sources to add or split inputs, configure join operations, and choose pre-projection join keys.
  4. Use Fields to include/exclude projected fields and change display labels.
  5. Save and reopen the dataset editor.
- Expected:
  - field selections, aliases, joins, and display labels persist after save/reopen
  - source-level latest/earliest/all selectors are not present
- Acceptance check:
  - pass when selected fields remain stable and the editor reloads the saved definition instead of regenerating default selections
- Evidence location:
  - `end2end/tests/datasets.spec.ts`
  - `cargo test -p tessara-api`

#### Dataset Aggregation And SQL Preview
- Role: admin
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `POST /api/admin/datasets/sql-preview`
- Steps:
  1. In Aggregation, switch between None, Row, and Field.
  2. Add grouping fields through the searchable combobox.
  3. In Row mode, add multiple sort fields and choose the shared direction.
  4. In Field mode, add metric rows and choose eligible source fields.
  5. Expand Generated SQL and confirm it refreshes from the current settings.
- Expected:
  - SQL uses `(form_version_id, field_id)` joins for form field values, avoids `field_key` as identity, and avoids ranked CTEs when no row picking is configured
  - unsupported aggregate-field combinations are not offered in the UI
- Acceptance check:
  - pass when SQL preview compiles from unsaved editor settings and reflects current source aliases, selected fields, joins, and aggregation
- Evidence location:
  - `end2end/tests/datasets.spec.ts`
  - `cargo test -p tessara-api`

#### Dataset Visibility And Role Gating
- Role: admin and scoped reader
- Paths:
  - `http://localhost:8080/datasets/{dataset_id}/edit`
  - `GET /api/datasets`
  - `GET /api/datasets/{dataset_id}/table`
- Steps:
  1. In the editor, search the Visibility tree.
  2. Toggle a node, a node with parents, and a node with descendants.
  3. Sign in as a scoped reader and request a dataset visible to that scope.
  4. Sign in as a no-access user and request dataset APIs.
- Expected:
  - visibility tree selection is ID-based and highlights search matches
  - scoped readers can access visible datasets and their full materialized output
  - no-access users cannot see dataset navigation and cannot fetch dataset APIs
- Acceptance check:
  - pass when dataset visibility gates dataset access and no implicit `__node_id` row filtering hides rows
- Evidence location:
  - `end2end/tests/permissions.spec.ts`
  - `end2end/tests/datasets.spec.ts`

### Acceptance Mapping

- Exit condition:
  - Admin can create a dataset, open detail, preview rows, edit the definition, and see the updated definition.
- Manual demonstration:
  - Dataset Authoring Editor
- Automated check:
  - `npx playwright test` from `end2end`, especially `end2end/tests/datasets.spec.ts`

- Exit condition:
  - Admin can configure data sources, projected fields, grouping/aggregation, generated SQL preview, and visibility in the Dataset editor.
- Manual demonstration:
  - Dataset Authoring Editor; Dataset Aggregation And SQL Preview; Dataset Visibility And Role Gating
- Automated check:
  - `end2end/tests/datasets.spec.ts`
  - `POST /api/admin/datasets/sql-preview` assertions in Playwright

- Exit condition:
  - Scoped readers see only datasets visible to their scope and can read the full materialized output for those datasets.
- Manual demonstration:
  - Dataset Visibility And Role Gating
- Automated check:
  - `end2end/tests/permissions.spec.ts`

- Exit condition:
  - No-capability users cannot see dataset navigation and cannot fetch dataset APIs.
- Manual demonstration:
  - Dataset Visibility And Role Gating
- Automated check:
  - `end2end/tests/permissions.spec.ts`

- Exit condition:
  - Dataset directory and preview surfaces use standard searchable/paginated table behavior and mobile cards.
- Manual demonstration:
  - Dataset Directory And Detail
- Automated check:
  - `end2end/tests/datasets.spec.ts`

- Exit condition:
  - Form-version render endpoint requires readable/manageable form access.
- Manual demonstration:
  - Dataset Authoring Editor
- Automated check:
  - `cargo test -p tessara-api`

- Exit condition:
  - Sprint closeout uses a squashed migration baseline.
- Manual demonstration:
  - inspect `crates/tessara-api/migrations`
- Automated check:
  - `.\scripts\local-launch.ps1 -FreshData` applies the single `001_baseline.sql` baseline

## 2026-06-06 - Sprint 3A Dataset Authoring Foundation Kickoff

- Kickoff status:
  - started Sprint 3A from clean `main` after committing and pushing roadmap sequencing prep
  - created branch `codex/sprint-3a` and worktree `C:\Users\eric-dev\Projects\tessara-sprint-3a`
  - added sprint plan at `docs/sprints/sprint-3a-plan.md`
- Planned verification:
  - `cargo fmt --all`
  - `.\scripts\validate.ps1`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo audit`
  - `npm --prefix end2end test`
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate implementation focus:
  - replace `/datasets` placeholders with native dataset directory/detail/create/edit/preview flows
  - keep authoring admin-only and preserve scoped read/preview behavior
  - tighten form-version render access before using it for dataset field options

## 2026-06-06 - Sprint 2F Closeout

- Completed:
  - closed the Operations Status implementation for local review at `/operations`
  - retained `operations:view` as a read-only capability separate from `analytics:refresh` and mutation permissions
  - updated Operations metrics to focus on actionable exceptions: open workflow assignments, draft form responses, datasets needing attention, and reporting data status
  - added standard searchable/filterable/paginated table behavior, mobile card rendering, and first-column links for workflow assignments and datasets
  - added cleanup for `pw-permissions-*` Playwright entities before and after the permissions suite
  - updated smoke and UAT scripts for the current native shell/API contracts
  - completed Orpheum `implementation-and-release-prep` artifact checks
  - marked Sprint 2F complete and Sprint 3A Dataset Authoring Foundation next in `docs/roadmap.md` after moving scoped analytics hardening behind real dataset/component/dashboard surfaces
- Validation:
  - `cargo fmt --all -- --check` - passed
  - `.\scripts\validate.ps1` - passed
  - `cargo test -p tessara-api` - passed through `validate.ps1`; direct non-DB run also passed
  - `cargo test -p tessara-web` - passed
  - `npm --prefix end2end test` - passed, 26/26
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"` - passed, 26/26
  - `.\scripts\smoke.ps1` - passed
  - `.\scripts\local-launch.ps1` - passed; stack rebuilt and seeded
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed
  - `orpheum check run --json` - passed all artifact-backed checks; Orpheum v1 boundary checks remained `not_evaluable_in_v1`
  - `git diff --check` - passed with CRLF normalization warnings for PowerShell scripts
  - `cargo audit --quiet` - passed; `RUSTSEC-2023-0071` is ignored because SQLx MySQL is optional and unreachable in the Postgres-only dependency tree, and `paste` remains an allowed upstream warning via Leptos/DataFusion
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed after direct API lint fixes and explicit web-crate allowances for existing Leptos/style lint debt
- Known release conditions:
  - Audit still prints the allowed `paste` unmaintained warning from upstream Leptos/DataFusion dependencies.
  - Web clippy cleanliness is enforced through documented crate-level allowances until the broader frontend patterns are refactored.
- Sprint Handoff / Demo Instructions:
  - local review stack is running at `http://localhost:8080`
  - sign in as `admin@tessara.local` / `tessara-dev-admin`
  - open `/operations` and confirm Operations appears in nav, status cards are action-oriented, workflow and dataset tables search/filter/page correctly, and row links navigate to the corresponding workflow-assignment filter or dataset detail
  - sign in as `operator@tessara.local` / `tessara-dev-operator` to verify scoped Operations visibility
  - sign in as `respondent@tessara.local` / `tessara-dev-respondent` to verify Operations is not visible
- Acceptance Mapping:
  - API access is protected by `operations:view`
  - admin sees global status data; scoped operators see scoped rows only
  - analytics status is no longer anonymous
  - Operations UI is native, read-only, and uses standard table controls
  - Playwright cleanup removes generated `pw-permissions-*` entities
- Next Focus:
  - Sprint 3A: Dataset Authoring Foundation Slice
  - future planning item: administrative workflow assignment detail page with reassignment, admin completion, deactivation, and capability decisions

## 2026-06-05 - Sprint 2F Operations Status Implementation

- Completed:
  - applied Orpheum `implementation-and-release-prep`; active session is `sess_20260605_210732_b03e9afacb284734ae3266e175409474`
  - locked the Sprint 2F implementation decision that Operations is a read-only `/operations` surface guarded by `operations:view`
  - added `operations:view` to the capability catalog and seeded it into the built-in admin and operator roles
  - added `GET /api/operations/status` with scoped workflow assignment and dataset readiness data
  - protected existing analytics status reads behind `operations:view` while keeping `analytics:refresh` separate
  - added native Operations navigation and route rendering for workflow assignment overview, dataset readiness, and reporting data status
  - extended API and Playwright permission coverage around operations visibility and scoped data containment
- Verification:
  - `cargo fmt --all` - passed
  - `cargo check --workspace` - passed
  - `cargo test -p tessara-api --no-run` - passed
  - focused operations API tests compiled and executed, but skipped DB assertions because `TEST_DATABASE_URL` was not set in this shell
  - `npm --prefix end2end ci` - passed
  - `npm --prefix end2end test -- --list` - passed and listed the new Operations visibility scenario
  - `git diff --check` - passed
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` - blocked by existing `tessara-web` clippy warnings outside the Operations slice

## 2026-06-05 - Sprint 2F Kickoff Preparation

- Completed:
  - created Sprint 2F branch `codex/sprint-2f` and worktree `C:\Users\eric-dev\Projects\tessara-sprint-2f` from clean `main`
  - reconciled the post-Sprint 2E Rust/UI styling detour as complete and moved the roadmap `(Next)` marker to Sprint 2F
  - wrote `docs/sprints/sprint-2f-plan.md` as the implementation contract for Runtime Status And Materialization
  - initialized Orpheum project support, configured the local catalog at `C:\Users\eric-dev\Projects\orpheum`, and mapped Sprint 2F to the `delivery-slice-planning`, `implementation-and-release-prep`, and `verification-and-release-gate` scenarios
  - applied the Orpheum `delivery-slice-planning` scenario for Sprint 2F kickoff scoping; active session is `sess_20260605_201224_d2175270ad3445b1b8f35e8d4ce850ee`
  - ran the initial Orpheum check; it failed because the full planning package artifacts were not authored yet
  - backfilled the lightweight Orpheum planning package across product, architecture, planning, verification, and security/compliance docs
  - reran `orpheum check run --json`; all artifact-backed checks passed, with only Orpheum v1 boundary/traceability checks marked `not_evaluable_in_v1`
  - finalized and closed the Orpheum `delivery-slice-planning` session; the next implementation pass should apply `implementation-and-release-prep`
- Planned verification:
  - `cargo fmt --all`
  - `.\scripts\validate.ps1`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo audit`
  - `npm --prefix end2end test`
  - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `orpheum check run --json` for any applied Sprint 2F Orpheum scenario
- Immediate implementation focus:
  - inventory existing workflow runtime, submission, dataset, component, dashboard, maintenance, and refresh paths before building the smallest native operator monitoring surface
  - decide route ownership and DTO boundaries, then apply Orpheum `implementation-and-release-prep` before broader code changes so implementation evidence is tracked from the start

## 2026-05-20 - Pre-RBAC Follow-Up Reconciliation

- Completed:
  - checked local `main` against `origin/main`; local history is clean and ahead with the native UI refresh follow-up commits
  - confirmed generated form assignment now flows through generated single-form workflows and normal workflow assignment mechanics
  - confirmed workflow assignments are the response-start and submission-access source of truth
  - confirmed stale auth/submission fields, legacy form-assignment paths, workflow availability summaries, workflow-mediated shortcut coverage, and standard validation wrappers have landed
  - updated roadmap and UI reset notes so remaining pre-RBAC work no longer repeats landed workflow/form assignment cleanup
- Remaining before user/RBAC overhaul:
  - finish residual Rust/UI table, chip, icon-action, and form-action spacing polish
  - decide and document the stylesheet path: keep `style/main.css` as the explicit delivery entrypoint or introduce the documented SCSS partial structure
  - add lightweight deployed-CSS selector verification to the validation story
  - keep migration consolidation as a separate cleanup decision rather than blocking the RBAC design unless the team chooses to reset the database baseline first

## 2026-05-06 - Sprint 2E Closeout

- Completed:
  - marked Sprint 2E complete in the roadmap and moved the `(Next)` marker to the post-Sprint 2E Rust/UI styling detour
  - delivered multi-step workflow authoring where each ordered step can reference a published version from a different form
  - allowed form versions to be reused across workflows rather than owned by a single workflow
  - added lineage-aware publish and assignment validation so composed workflow forms stay on one node line without sibling branching
  - added contextual workflow assignment candidates, assignee filtering, and idempotent bulk assignment creation
  - delivered same-assignee automatic runtime handoff from one workflow step to the next, with completion state persisted for workflow instances
  - refreshed workflow, organization, home, and response surfaces to expose Sprint 2E workflow context while deferring further UX polish to the Rust/UI detour
- Validation:
  - `.\scripts\local-launch.ps1` - passed; rebuilt and reseeded the local stack
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` - passed
  - `.\scripts\smoke.ps1` - passed after updating demo seed assertions for the six published Sprint 2E forms
  - `cargo test -p tessara-api` - passed
  - `cargo test -p tessara-web` - passed
  - `cd end2end; npx playwright test` - passed, 33/33
  - `cargo fmt --all` - passed
  - `.\scripts\local-launch.ps1 -SkipBuild` - passed; local review stack left running at `http://localhost:8080`
- Next Focus:
  - Post-Sprint 2E Design Detour: Rust/UI Styling And Component Alignment

## Sprint Handoff / Demo Instructions

### Multi-Step Workflow Authoring
- Role: admin
- Paths:
  - `http://localhost:8080/app/workflows/new`
  - `http://localhost:8080/app/workflows/{workflow_id}/edit`
  - `POST /api/workflows/{workflow_id}/versions`
  - `POST /api/workflow-versions/{workflow_version_id}/publish`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app/workflows/new`, create workflow metadata, and use the version editor to add two or more ordered steps.
  3. Select published form versions from different forms, including Program/Activity combinations or the sibling Activity demo forms.
  4. Save as draft or publish the workflow version.
- Expected:
  - each step has its own linked published form version, form versions can be reused across workflows, and invalid sibling-branch combinations are rejected before publish.
- Acceptance check:
  - Pass when the workflow publishes only for a straight-line node lineage and rejects missing, unpublished, or incompatible step collections with stable validation feedback.
- Evidence location:
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `npx playwright test`

### Contextual Workflow Assignment
- Role: admin
- Paths:
  - `http://localhost:8080/app/workflows/assignments`
  - `http://localhost:8080/app/organization/{node_id}`
  - `GET /api/workflow-assignment-candidates`
  - `GET /api/workflow-assignment-candidates/assignees?workflow_version_id=...&node_id=...`
  - `POST /api/workflow-assignments/bulk`
- Steps:
  1. Open an organization node detail route and use `Assign Workflow`, or open `/app/workflows/assignments`.
  2. Choose a valid `Node path - Workflow` candidate.
  3. Select one or more valid assignees and create assignments.
  4. Repeat the same assignment request for the same assignee.
- Expected:
  - candidates respect assigner scope and workflow step-form lineage, assignees are filtered to valid users, and bulk creation reports created/reactivated/skipped outcomes without duplicates.
- Acceptance check:
  - Pass when invalid workflow/node/assignee combinations are unavailable or rejected, and repeat bulk assignment is idempotent.
- Evidence location:
  - `cargo test -p tessara-api`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `.\scripts\smoke.ps1`

### Step Runtime Handoff
- Role: respondent
- Paths:
  - `http://localhost:8080/app`
  - `http://localhost:8080/app/responses`
  - `POST /api/workflow-assignments/{workflow_assignment_id}/start`
  - `POST /api/submissions/{submission_id}/submit`
- Steps:
  1. Sign in as the assigned response user.
  2. Start the pending workflow work from Home or Responses.
  3. Complete and submit step 1.
  4. Confirm the next step becomes the active work item and opens with its own form.
  5. Complete the final step.
- Expected:
  - the workflow advances in order for the same assignee, each step uses its selected form version, and the workflow instance is marked complete after the final submit.
- Acceptance check:
  - Pass when step 2 cannot be started before step 1, step 1 submit activates step 2, and final submit leaves completed history visible.
- Evidence location:
  - `cargo test -p tessara-api`
  - `npx playwright test`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

### Backward Compatibility And Native Routes
- Role: admin, respondent
- Paths:
  - `http://localhost:8080/app/workflows`
  - `http://localhost:8080/app/organization`
  - `http://localhost:8080/app/responses`
  - existing single-step workflow assignment/start/draft/submit/review APIs
- Steps:
  1. Open the workflow directory and inspect existing single-step demo workflows.
  2. Start, save, submit, and review a single-step response item.
  3. Refresh touched workflow, organization, home, and response routes.
- Expected:
  - existing single-step workflows continue to work through the compatibility anchors, touched routes remain native SSR/hydrated, and runtime UI displays workflow context without breaking older records.
- Acceptance check:
  - Pass when single-step response lifecycle behavior remains available and touched app routes load cleanly after refresh.
- Evidence location:
  - `cargo test -p tessara-web`
  - `npx playwright test`
  - `.\scripts\smoke.ps1`

### Deferred Rust/UI Styling Detour
- Role: admin
- Paths:
  - `docs/roadmap.md`
  - `docs/sprints/sprint-2e-plan.md`
- Steps:
  1. Review the roadmap section after Sprint 2E.
  2. Confirm the UX feedback collected during Sprint 2E is assigned to the Rust/UI styling detour instead of remaining as unfinished Sprint 2E scope.
- Expected:
  - Sprint 2E closes as functionally complete while table controls, icon polish, form spacing, assignee chips, stylesheet organization, and broader Rust/UI component adoption are tracked as next work.
- Acceptance check:
  - Pass when the roadmap names the detour and keeps Sprint 2F available after the styling/design alignment work.
- Evidence location:
  - `docs/roadmap.md`
  - `docs/sprints/sprint-2e-plan.md`

## Acceptance Mapping

- Exit condition:
  - A tester can create a workflow with more than one step.
- Manual demonstration:
  - `Multi-Step Workflow Authoring`
- Automated check:
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `npx playwright test`

- Exit condition:
  - Steps can reference different forms and reusable form versions.
- Manual demonstration:
  - `Multi-Step Workflow Authoring`
- Automated check:
  - `cargo test -p tessara-api`

- Exit condition:
  - Assignment works from both an organization node and the global assignment console using only valid node/workflow/assignee combinations.
- Manual demonstration:
  - `Contextual Workflow Assignment`
- Automated check:
  - `cargo test -p tessara-api`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

- Exit condition:
  - Starting and submitting step 1 activates the next step as the active work item through the UI.
- Manual demonstration:
  - `Step Runtime Handoff`
- Automated check:
  - `cargo test -p tessara-api`
  - `npx playwright test`

- Exit condition:
  - Touched workflow, organization, home, and response routes remain native, refresh-safe, and compatible with existing single-step behavior.
- Manual demonstration:
  - `Backward Compatibility And Native Routes`
- Automated check:
  - `cargo test -p tessara-web`
  - `.\scripts\smoke.ps1`

- Exit condition:
  - Deferred UX work is captured without making the later one-draft/one-active/retired lifecycle or delegated Home redesign harder.
- Manual demonstration:
  - `Deferred Rust/UI Styling Detour`
- Automated check:
  - roadmap/progress documentation review in this closeout entry

## 2026-05-06 - Sprint 2E Rust/UI Table Pivot

- Completed:
  - pivoted the Sprint 2E workflow directory and assignment directory tables toward Rust/UI-style component structure instead of introducing DataTables.net
  - added native table controls for search, status filtering, sorting, page size, row count summaries, and previous/next pagination
  - kept workflow directory action buttons spaced at 6px, in a single row on wide screens and a single column on narrower screens
  - added a future-work note to schedule a dedicated post-Sprint 2E design detour for adopting Rust/UI-style components across the broader application
  - expanded the Rust/UI design-detour note to include stylesheet consolidation, including a single documented app stylesheet entrypoint, named SCSS partials, clarified handling for the parallel `crates/tessara-web/assets/base.css` path, and deployed-CSS selector verification
- Validation:
  - `cargo fmt --all`
  - `cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api`
  - `.\scripts\local-launch.ps1`
- Remaining:
  - run Playwright, smoke, and UAT coverage before final Sprint 2E closeout

## 2026-05-05 - Sprint 2E Concrete Lineage Demo Forms

- Completed:
  - added two published Activity-scoped demo forms linked to sibling Activities: `Demo Intake Activity Checkpoint` on `Demo Activity Intake and Orientation` and `Demo Workshop Activity Checkpoint` on `Demo Activity Family Workshops`
  - expanded form version summaries with linked assignment-node context so workflow authoring can reason about concrete nodes, not only node types
  - updated workflow step option filtering so a first Activity-linked form excludes sibling Activity-linked forms while still allowing ancestors and descendants on the same line
  - tightened publish validation so sibling concrete form-linked nodes are rejected even when their forms share the same node type
  - updated runtime step-node resolution and assignment candidates to prefer a form version's linked nodes before falling back to node-type compatibility
  - updated demo seed expectations from four to six forms and added regression coverage for sibling Activity workflow rejection
- Validation:
  - `cargo fmt --all`
  - `cargo check -p tessara-api`
  - `cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api workflow_publish_rejects_sibling_step_assignment_nodes -- --nocapture`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api demo_seed_creates_full_uat_dataset_and_is_repeatable -- --nocapture`
  - `.\scripts\local-launch.ps1`
- Remaining:
  - run full API, Playwright, smoke, and UAT coverage before Sprint 2E closeout

## 2026-05-04 - Sprint 2E Workflow UX Feedback Pass

- Completed:
  - removed the visible legacy Linked Form selector from workflow create/edit because forms are now selected per workflow step
  - changed new workflow-version authoring to create and publish in one action from the operator's point of view
  - made workflow metadata create/update payloads tolerate an omitted legacy form anchor while preserving backward compatibility
  - preselect assignment candidates when arriving with a node/workflow context, moved assignees below the candidate picker, and made the multiselect taller
  - refreshed assignment results in place after bulk creation and converted the assignment directory to a table
  - redirected response users directly into the next workflow step after submit when one is available
  - added a server-side start guard so later workflow steps cannot be started before the previous step is completed
- Validation:
  - `cargo fmt --all`
  - `cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api multi_step_workflow_ -- --nocapture`
  - `cargo test -p tessara-api --no-run`
  - `cargo test -p tessara-web --no-run`
- Remaining:
  - run full API, Playwright, smoke, and UAT coverage before Sprint 2E closeout

## 2026-05-04 - Sprint 2E Lineage-Pinned Workflow Form Collections

- Completed:
  - refined composed workflow validation so every step form scope must stay on one straight node-type lineage with no branching
  - pinned assignment candidates to the highest/broadest component form scope, so Program plus Activity workflows assign at Program, not Partner
  - kept descendant step execution intact: a Program-assigned workflow can start an Activity-scoped step against the matching descended Activity node
  - filtered workflow authoring form-version choices client-side after step selection so later steps stay within the selected lineage
  - applied assignment-candidate validation to direct create/update paths, not only candidate and bulk assignment flows
  - added regression coverage for Program-to-Activity workflow execution, highest-scope candidate pinning, and branching publish rejection
- Validation:
  - `cargo fmt --all`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api multi_step_workflow_ -- --nocapture`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api workflow_publish_rejects_branching_step_form_scopes -- --nocapture`
  - `cargo test -p tessara-api --no-run`
  - `cargo test -p tessara-web --no-run`
  - `cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate`
- Remaining:
  - run full API, Playwright, smoke, and UAT coverage before Sprint 2E closeout

## 2026-05-04 - Sprint 2E Descendant Step Scope Compatibility

- Completed:
  - replaced the incorrect single-node-type publish rule with composed step validation that allows different form scopes in one workflow version
  - changed workflow assignment candidates to accept a node when every step form can resolve to that node or one of its descendants
  - changed step start runtime so each submission uses the resolved node for that step's form while the workflow assignment/instance remains attached to the workflow node
  - fixed pending-card and submission runtime step counts to count the full workflow version instead of only the currently selected row
  - moved multi-step test respondent login after demo seeding and added a Program-to-Activity descendant-scope regression test
  - redeployed the local app and confirmed the previously failing draft workflow version publishes successfully
- Validation:
  - `cargo fmt --all`
  - `TEST_DATABASE_URL=postgres://tessara:tessara@localhost:5432/tessara_test cargo test -p tessara-api multi_step_workflow_ -- --nocapture`
  - `cargo test -p tessara-api --no-run`
  - `cargo test -p tessara-web --no-run`
  - `cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate`
  - `.\scripts\local-launch.ps1 -SkipSeed`
- Remaining:
  - run full API, Playwright, smoke, and UAT coverage before Sprint 2E closeout

## 2026-05-04 - Sprint 2E Workflow Step Authoring Controls

- Completed:
  - replaced the fixed three-row workflow version editor with dynamic step rows
  - added authoring controls to add, remove, move up, and move down workflow steps before creating a workflow version
  - changed step collection to read the current DOM order so submitted step position matches the operator-authored order
- Validation:
  - `cargo fmt --all`
  - `cargo test -p tessara-web --no-run`
  - `cargo test -p tessara-api --no-run`
- Remaining:
  - add Playwright coverage for add/remove/reorder and run browser verification against a local app instance

## 2026-05-04 - Sprint 2E Reusable Form Workflow Ingredients

- Completed:
  - expanded Sprint 2E scope so a form and any published form version can be included in any number of workflows
  - dropped the remaining `workflows.form_id` exclusivity in the Sprint 2E migration, alongside the existing `workflow_versions.form_version_id` uniqueness removal
  - removed API validation that enforced one workflow per form while preserving unique workflow slugs
  - tightened the legacy published-form compatibility helper so it updates only the generated default workflow/version instead of grabbing any workflow version that uses the same form version
  - updated form detail reads so workflows are shown when a form appears through any workflow step, not only through the legacy workflow anchor
  - added API regression coverage for reusing the same form version across multiple workflows and for showing step-form workflow usage from form detail
- Validation:
  - `cargo fmt --all`
  - `cargo test -p tessara-api --no-run`
  - `cargo test -p tessara-api form_versions_can_be_reused_across_workflows -- --nocapture`
- Remaining:
  - run the full DB-backed API suite, Playwright flow, smoke, local launch, and UAT once the broader Sprint 2E polish pass is complete

## 2026-05-04 - Sprint 2E Kickoff

- Completed:
  - confirmed clean `main` and selected `Sprint 2E: Multi-Step Workflow Authoring And Execution` from the single roadmap `(Next)` marker
  - created branch `codex/sprint-2e` with worktree `C:\Users\ericw\Projects\tessara-sprint-2e`
  - added the Sprint 2E plan at `C:\Users\ericw\Projects\tessara-sprint-2e\docs\sprints\sprint-2e-plan.md`
- Validation Plan:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cd end2end; npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate Focus:
  - implement workflow versions as ordered form collections with same-assignee automatic step handoff and shared assignment eligibility APIs

## 2026-05-04 - Sprint 2D Closeout

- Completed:
  - marked Sprint 2D complete in the roadmap and moved the `(Next)` marker to Sprint 2E
  - delivered assigned response starts, draft save/resume, strict submit, and submitted read-only review through native `/app/responses*` UI
  - preserved the existing public response routes and endpoints while moving touched lifecycle SQL/orchestration into submissions repo/service helpers
  - kept browser response flows config-aware for custom cookie names and reserved bearer handling for explicit API/script/test flows
  - polished the response queue into separate `Assigned Work`, `Draft Queue`, and `Submitted Work` tables, with workflow descriptions in the `Details` column and 16px login action spacing
- Validation:
  - `.\scripts\local-launch.ps1` passed and rebuilt/reseeded the local stack
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` passed
  - `.\scripts\smoke.ps1` passed: 23 passed, 0 failed, with the demo-flow JSON probe passing
  - `.\scripts\local-launch.ps1 -SkipBuild` passed after smoke teardown and left the app running at `http://localhost:8080/app`
  - `cargo fmt --all` passed
  - `cargo test -p tessara-submissions` passed: 3 passed, 0 failed
  - `cargo test -p tessara-api` passed across API, workflow runtime, auth, and response lifecycle coverage
  - `cargo test -p tessara-web` passed: 18 passed, 0 failed
  - `cd end2end; npx playwright test` passed: 33 passed, 0 failed
- Next Sprint:
  - Sprint 2E: Multi-Step Workflow Authoring And Execution

## Sprint Handoff / Demo Instructions

### Assigned Start And Draft Resume
- Role: respondent
- Paths: `/app/responses`, `/app/responses/{submission_id}/edit`, `/api/workflow-assignments/{assignment_id}/start`, `/api/submissions?status=draft`
- Walkthrough:
  - sign in as a response user, open `/app/responses`, and start assigned work when an assigned workflow is present
  - enter values, save the draft, return to `/app/responses`, and reopen the same draft from `Draft Queue`
  - confirm the values are still present and the queue points back to the same in-progress submission
- Expected result:
  - draft save preserves values and audit history, no duplicate draft is created, and resume returns to the same response item
- Automated evidence:
  - `response_lifecycle_saves_resumes_submits_and_locks_submitted_records`
  - `assignee pending start opens the matching draft directly and removes it from pending after submit`
  - `draft save preserves values for later resume`
  - `draft resume actions resolve to the same in-progress response item`

### Strict Submit And Read-Only Review
- Role: respondent
- Paths: `/app/responses/{submission_id}/edit`, `/app/responses/{submission_id}`, `/api/submissions/{submission_id}/submit`
- Walkthrough:
  - open a draft response, leave a required value blank, and attempt submit
  - confirm the UI shows the server-backed validation feedback and the response remains in `Draft Queue`
  - fill the required values, submit once, and land on the submitted read-only review page
  - try edit/save/resubmit/delete paths against the submitted response
- Expected result:
  - invalid submit stays draft; valid submit sets `submitted_at`, completes the linked workflow step, audits submit, removes the item from active queues, and keeps review read-only
- Automated evidence:
  - `submit feedback keeps incomplete responses in draft`
  - `response_lifecycle_saves_resumes_submits_and_locks_submitted_records`
  - `submitted_responses_reject_edit_save_resubmit_and_delete`

### Scoped Review And Delegation
- Roles: admin, scoped operator, respondent, delegator
- Paths: `/app/responses`, `/api/submissions`, `/api/submissions/{submission_id}`, delegated response context with `delegateAccountId`
- Walkthrough:
  - confirm an admin can review all submitted responses
  - confirm a scoped operator can review only in-scope submissions and is denied for out-of-scope records
  - confirm response users see their own work and delegators can query accessible delegated work
- Expected result:
  - review access follows the admin, scoped operator, response user, and delegation boundaries without leaking out-of-scope submissions
- Automated evidence:
  - `scoped_operator_cannot_review_out_of_scope_submission_by_uuid`
  - `delegator_can_query_pending_work_for_an_accessible_delegate_account`
  - response route Playwright coverage for role-gated manual start and accessible response queues

### Native Response UI And Hydration
- Roles: admin, respondent
- Paths: `/app/responses`, `/app/responses/new`, `/app/responses/{submission_id}`, `/app/responses/{submission_id}/edit`, `/app/login`
- Walkthrough:
  - refresh the response routes and inspect the separate `Assigned Work`, `Draft Queue`, and `Submitted Work` tables
  - confirm rows keep work text on the left, workflow descriptions in `Details`, and stacked actions on narrower widths
  - confirm submitted rows expose `View` only and the sign-in form keeps a 16px gap above the submit button
  - check browser console output during route load and hydration
- Expected result:
  - touched `/app/responses*` surfaces stay native SSR-owned, hydrate cleanly, remain console-clean, and do not introduce bridge-backed behavior
- Automated evidence:
  - `responses route stays readable and console-clean on the native shell`
  - `response users are redirected away from the manual start screen while admins keep it`
  - `cargo test -p tessara-web`

## Acceptance Mapping

- Draft save preserves values and audit history, and resume returns to the same in-progress submission:
  - demonstrated through the respondent draft walkthrough and covered by response lifecycle API and Playwright draft resume tests
- Submit fails visibly and leaves the submission as draft when required values are missing or invalid:
  - demonstrated by the missing-required submit walkthrough and covered by strict submit API tests plus Playwright feedback coverage
- Submit succeeds only once, sets `submitted_at`, completes the linked workflow step, records audit, removes the item from pending/draft queues, and shows read-only review:
  - demonstrated through the valid submit walkthrough and covered by lifecycle API tests and pending-start Playwright coverage
- Submitted responses cannot be edited, saved, resubmitted, or deleted:
  - demonstrated by submitted-route checks and covered by immutability API tests
- Admins can review all responses; scoped operators only see in-scope responses; response users and delegators only see their own or accessible delegated work:
  - demonstrated through scoped review and delegation walkthroughs and covered by operator-scope and delegation API tests
- Touched `/app/responses*` surfaces stay native SSR-owned, hydrate cleanly, and do not add new bridge-backed behavior:
  - demonstrated by route refresh and console checks and covered by web unit tests plus Playwright console-clean response route coverage
- Existing public routes and endpoints remain the response lifecycle surface:
  - verified through preserved `/app/responses*`, `/api/responses/options`, `/api/submissions*`, and workflow-assignment start coverage in API, Playwright, smoke, and UAT runs

## 2026-05-04 - Sprint 2D Response Lifecycle Implementation

- Completed:
  - moved response lifecycle orchestration into the submissions service and response persistence SQL into repository helpers
  - switched touched response and workflow-start browser endpoints to the config-aware authenticated request extractor
  - tightened submit validation so missing, empty, invalid, non-draft, and already-submitted transitions are rejected before final lifecycle changes
  - polished draft save/resume, submit feedback, and submitted read-only review behavior in the native Responses UI
  - added API and Playwright coverage for Sprint 2D lifecycle behavior
- Validation:
  - `cargo fmt --all` passed
  - `cargo test -p tessara-submissions` passed
  - `cargo test -p tessara-api` passed
  - `cargo test -p tessara-web` passed
  - `cd end2end; npm ci` passed
  - `cd end2end; npx playwright test --list` passed and listed 33 tests
  - `.\scripts\local-launch.ps1 -FreshData` passed after Docker Desktop became available
  - `cd end2end; npx playwright test` passed: 33 passed, 0 failed
  - `.\scripts\smoke.ps1` passed: 23 passed, 0 failed, with demo-flow probe passing
  - `.\scripts\local-launch.ps1 -FreshData -SkipBuild` relaunched the rebuilt image and reseeded demo data after smoke teardown
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` passed
- Next Focus:
  - keep the deployment open for reviewer UAT and prepare the branch for review

## 2026-05-04 - Sprint 2D Kickoff

- Completed:
  - confirmed clean `main` and selected `Sprint 2D: Draft, Submit, And Review Response Slice` from the single roadmap `(Next)` marker
  - created branch `codex/sprint-2d` with worktree `C:\Users\ericw\Projects\tessara-sprint-2d`
  - added the Sprint 2D plan at `C:\Users\ericw\Projects\tessara-sprint-2d\docs\sprints\sprint-2d-plan.md`
- Validation Plan:
  - `cargo fmt --all`
  - `cargo test -p tessara-submissions`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cd end2end; npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate Focus:
  - make the respondent response lifecycle coherent: save draft, resume with values intact, strict submit, and submitted read-only review through native `/app/responses*`

## 2026-05-04 - Sprint 2E Contextual Assignment UX Scope Added

- Decision:
  - added contextual workflow assignment UX to Sprint 2E planning because multi-step workflow assignment semantics are already scheduled there
  - selected two equal-priority entry points: organization-node `Assign Workflow` and the global assignment console
  - selected strict eligibility: normal UI choices should only show valid node/workflow candidates and valid assignees
- Documentation:
  - updated `docs/roadmap.md` so Sprint 2E includes shared assignment-candidate APIs, node-context assignment, global `Node path - Workflow` selection, and assignee multiselect behavior
  - updated `docs/tessara-roadmap-backlog-github-ready-described.md` with new issue `2E-07: Contextual workflow assignment UX and eligibility APIs`
  - renumbered the Sprint 2E UAT issue to `2E-08` and updated immediate Sprint 2F dependencies accordingly

## 2026-05-04 - Sprint 2C Closeout

- Completed:
  - completed full teardown and redeploy from the Sprint 2C verification mirror, including Compose shutdown, Postgres volume refresh, API/app image rebuild, container recreation, and UAT demo reseed
  - marked Sprint 2C complete in `docs/roadmap.md` and moved the `(Next)` marker to Sprint 2D
  - preserved public workflow/submission HTTP contracts while splitting workflow and submission backend code into `dto`, `handlers`, `service`, and `repo` modules
  - hardened workflow-assignment start authorization so admins can start any assignment, scoped operators can start only assignments inside effective scope, and response users stay limited to their own or delegated work
  - stabilized native workflow/response SSR routes and updated Playwright expectations to the current UI shape
  - removed redundant main-page title/explanation blocks on touched shell routes, renamed the Responses surface to `Form Responses`, fixed response edit hydration after assignment start, documented the faster dev loop, and removed the accidental amber administration shell border
- Validation:
  - `.\scripts\local-launch.ps1 -FreshData` passed from `C:\Users\rdpuser\AppData\Local\Temp\tessara-sprint-2c-verify`
  - `cargo fmt --all` passed; the command still reports the known `could not canonicalize path C:\Users\ericw` warning
  - `cargo test -p tessara-api` passed with `CARGO_TARGET_DIR=C:\Users\rdpuser\AppData\Local\Temp\tessara-sprint-2c-target`; a first repo-local target attempt failed on the known `lzma-sys` MSVC archive path issue
  - `cargo test -p tessara-web` passed with the same temp target
  - `.\scripts\smoke.ps1` passed with the same temp target
  - `.\scripts\local-launch.ps1 -FreshData -SkipBuild` relaunched the rebuilt image and reseeded demo data after smoke teardown
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` passed
  - `cd end2end; npx playwright test` passed: 31 passed, 0 failed
- Next Sprint:
  - Sprint 2D: Draft, Submit, And Review Response Slice

## Sprint Handoff / Demo Instructions

### Native Workflow And Response Entry
- Role: admin
- Paths:
  - `http://localhost:8080/app/workflows`
  - `http://localhost:8080/app/workflows/assignments`
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open Workflows, then Workflow Assignments.
  3. Review assignment context and start or inspect response work from the native shell.
  4. Open Responses and confirm the page is titled `Form Responses`.
- Expected:
  - workflow and response pages render in the native Tessara shell, stay readable after refresh, and do not fall back to `/app/admin`.
- Acceptance check:
  - pass if workflow/response route ownership remains native and response work opens the correct draft/detail route.
- Evidence location:
  - `cd end2end; npx playwright test` tests 17, 18, 23, 24, 25, and 26.

### Scoped Assignment Start Authorization
- Role: operator
- Paths:
  - `POST /api/workflow-assignments/{assignment_id}/start`
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as a scoped operator.
  2. Start an assignment attached to an in-scope node.
  3. Attempt to start an assignment UUID outside the operator's effective node scope.
- Expected:
  - the in-scope start succeeds; the out-of-scope start returns `403` with code `forbidden`.
- Acceptance check:
  - pass if scoped operators cannot bypass UI scoping by posting another assignment UUID directly.
- Evidence location:
  - `cargo test -p tessara-api` includes `scoped_operator_cannot_start_out_of_scope_workflow_assignment_by_uuid`.

### Response Resume And Hydration Stability
- Role: respondent
- Paths:
  - `http://localhost:8080/app/responses`
  - `http://localhost:8080/app/responses/{submission_id}/edit`
- Steps:
  1. Sign in as `respondent@tessara.local`.
  2. Open Responses.
  3. Start a pending assignment-backed response.
  4. Submit or resume the resulting draft.
- Expected:
  - starting work opens the matching edit route, the form hydrates reliably, and submitted work leaves the pending queue.
- Acceptance check:
  - pass if the app reaches the response editor instead of getting stuck on `Loading response form...`.
- Evidence location:
  - `cd end2end; npx playwright test` tests 23 and 24.

### Main Navigation UI Cleanup
- Role: admin
- Paths:
  - `http://localhost:8080/app/organization`
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/administration`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open the main navigation pages.
  3. Confirm each page keeps the top shell title and no longer repeats a large redundant page heading/explanation block.
  4. Open Administration and confirm the shell border is neutral, not amber.
- Expected:
  - touched main navigation pages have less duplicated titling and Administration visually matches the neutral shell treatment.
- Acceptance check:
  - pass if main navigation pages are readable without duplicate hero titles and Administration has no yellow border highlight.
- Evidence location:
  - `cd end2end; npx playwright test` tests 5 through 18 and the final computed-style verification from the admin border fix.

## Acceptance Mapping

- Exit condition:
  - public workflow/submission API contracts remain compatible while backend code is decomposed.
- Manual demonstration:
  - Native Workflow And Response Entry.
- Automated check:
  - `cargo test -p tessara-api`; `cd end2end; npx playwright test`.

- Exit condition:
  - admins, scoped operators, respondents, and delegators observe correct assignment-start boundaries.
- Manual demonstration:
  - Scoped Assignment Start Authorization and Response Resume And Hydration Stability.
- Automated check:
  - `scoped_operator_cannot_start_out_of_scope_workflow_assignment_by_uuid`; Playwright response-start and draft-resume tests.

- Exit condition:
  - `/app/workflows*` and `/app/responses*` remain native SSR-owned and stable under refresh.
- Manual demonstration:
  - Native Workflow And Response Entry.
- Automated check:
  - Playwright workflow, response, deep-link, no-JavaScript, and protected-route tests.

- Exit condition:
  - Sprint 2C is deployable from a clean local Compose stack.
- Manual demonstration:
  - open `http://localhost:8080/app` after closeout redeploy.
- Automated check:
  - `.\scripts\local-launch.ps1 -FreshData`, `.\scripts\smoke.ps1`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.

## 2026-05-04 - Sprint 2C Local Dev Loop Adjustment

- Decision:
  - use the faster local development loop for Sprint 2C UI and Playwright iteration instead of defaulting to full Docker teardown/rebuild/redeploy
  - prefer host-run Tessara with Docker Postgres where possible, or `.\scripts\local-refresh-api.ps1` / `.\scripts\local-launch.ps1 -SkipBuild` when a container refresh is sufficient
  - reserve full teardown, rebuild, and redeploy for Docker/dependency/migration changes, release-build verification, smoke, manual UAT, and sprint closeout
- Validation context:
  - `.\scripts\local-launch.ps1 -FreshData -SkipBuild` refreshed the Compose data volume and relaunched the existing image in about 20 seconds during the Playwright retest loop
  - targeted Playwright response-start coverage passed after switching away from repeated full rebuilds
  - `.\scripts\local-refresh-api.ps1 -SkipSeed` rebuilt only the API/app image after the response edit-loader hardening, then `.\scripts\local-launch.ps1 -FreshData -SkipBuild` reset seeded data without rebuilding the image
  - `cd end2end; npx playwright test` passed: 31 passed, 0 failed

## 2026-05-04 - Sprint 2C Backend Decomposition And Runtime Hardening

- Completed:
  - refreshed Sprint 2C tracking around the current roadmap: renamed GitHub milestone `#14`, rescopied issue `#90` as `2C-06`, and created issues `#93` through `#99` for `2C-01` through `2C-05`, `2C-07`, and `2C-08`
  - split workflow and submission API modules into bounded `dto`, `handlers`, `service`, and `repo` modules while preserving public route registrations and endpoint names
  - moved submission access checks, field-contract loading, and submission audit writes behind service/repo helpers
  - closed the workflow-assignment start gap so scoped operators can only start assignments inside effective node scope while admins and valid self/delegation starts remain supported
  - added the negative workflow-runtime regression for an out-of-scope operator assignment-start UUID
  - added a non-inline `event_button` action primitive in `tessara-ui` for touched native SSR surfaces to use instead of raw inline handlers
- Validation:
  - `cargo fmt --all`
  - `cargo check -p tessara-api`
  - `cargo test -p tessara-api --test workflow_runtime scoped_operator_cannot_start_out_of_scope_workflow_assignment_by_uuid -- --nocapture --test-threads=1`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-ui`
- Docker Follow-Up:
  - confirmed Docker Desktop is now reachable (`docker info` reported server version `29.4.0`)
  - `.\scripts\smoke.ps1` passed after setting `CARGO_TARGET_DIR` to `C:\Users\rdpuser\AppData\Local\Temp\tessara-sprint-2c-target`; the default target under `C:\Users\ericw\Projects\tessara-sprint-2c\target` is not writable enough from this `rdpuser` session for the `lzma-sys` MSVC archive step
  - direct `.\scripts\local-launch.ps1` from the sprint worktree reached Docker but Docker could not evaluate the `C:\Users\ericw\Projects\tessara-sprint-2c` build context because the current session is running as `rdpuser`
  - mirrored the source, excluding `.git`, `target`, `tmp`, and prior Playwright output, to `C:\Users\rdpuser\AppData\Local\Temp\tessara-sprint-2c-verify`; `.\scripts\local-launch.ps1` passed there and served `http://localhost:8080/app`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` passed against the local Compose deployment
  - `cd end2end; npm ci; npx playwright install; npx playwright test` ran against the deployment: 22 passed, 9 failed on existing browser-suite expectations around theme popover interaction, current dashboard/response/dataset headings, response card CSS selectors, duplicate administration text, and hidden mobile search visibility
  - spot-checked the Sprint 2C response-start path in the live browser: respondent pending starts rendered and clicking `button[data-workflow-assignment-id]` opened `/app/responses/{submission_id}/edit`

## 2026-05-04 - Sprint 2C Kickoff

- Completed:
  - confirmed clean `main` and selected `Sprint 2C: Workflow/Response Backend Decomposition And Runtime Hardening Slice` from the single roadmap `(Next)` marker
  - created branch `codex/sprint-2c` with worktree `C:\Users\ericw\Projects\tessara-sprint-2c`
  - added the Sprint 2C plan at `C:\Users\ericw\Projects\tessara-sprint-2c\docs\sprints\sprint-2c-plan.md`
- Validation Plan:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cd end2end; npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate Focus:
  - refresh the stale Sprint 2C backlog and GitHub issue set, then begin workflow/submission backend decomposition and scoped assignment-start hardening

## 2026-05-03 - Audit Recommendation Roadmap Placement

- Completed:
  - added cross-cutting scope-regression and dependency-audit rules to `docs/roadmap.md`
  - kept the workflow assignment scope fix in Sprint 2C, where workflow/runtime backend decomposition is already next
  - placed cookie-helper cleanup and browser-token separation into Sprint 2D, where response flows continue the settled auth/session contract
  - added `Sprint 2G: Scoped Analytics And Reporting Compatibility Hardening` before Phase 3 so report execution, chart/component metadata, aggregation, and dashboard scope behavior are corrected before dataset/component authoring expands
  - carried scoped analytical visibility guarantees into Dataset, Component, Chart, and Dashboard roadmap slices
  - placed `cargo audit` enforcement and advisory policy in Sprint 2F with other CI/runtime gates
  - placed docs archive-reference cleanup in Sprint 6A and production CORS/browser-token hardening in Sprint 6B
  - updated `docs/README.md` so it no longer links to absent `docs/archive` paths
- Notes:
  - Sprint 2G is intentionally a compatibility-hardening slice, not a new long-term endorsement of `Report`, `Aggregation`, or `Chart` as target assets

## 2026-05-03 - Roadmap Reconciliation After UI Overhaul Detour

- Completed:
  - updated `docs/roadmap.md` to be authoritative as of May 3, 2026
  - marked Sprint 2B as complete based on the implemented auth/session hardening and native settled-surface work
  - added `UI Overhaul 2.0: Out-Of-Roadmap UX Detour` as a completed detour slice between Sprint 1G and Phase 2
  - refreshed the current baseline to reflect the approved shell posture, sidebar footer context, access-denied toast flow, queue-first home direction, organization explorer posture, native Components/Datasets inspection surfaces, and form section description/column-count support
  - reframed Sprint 2C as the next roadmap sprint around workflow/response backend decomposition and runtime hardening, since route ownership work was already pulled forward by Sprint 2A, Sprint 2B, and the UI detour
- Validation:
  - `.\scripts\local-launch.ps1`
  - `.\scripts\smoke.ps1 -ComposeApi -KeepServices`
  - roadmap consistency check confirmed a single `(Next)` marker on Sprint 2C
- Notes:
  - a follow-on audit placement pass now assigns those recommendations across Sprint 2C, Sprint 2D, Sprint 2F, Sprint 2G, Phase 3, Phase 4, Phase 5, Sprint 6A, and Sprint 6B

## 2026-04-19 - UI Overhaul 2.0 Kickoff

- Completed:
  - confirmed `main` was clean before creating the dedicated UI migration branch and worktree
  - created branch `codex/ui-overhaul-2-0` with sibling worktree `D:\Projects\tessara-ui-overhaul-2-0`
  - wrote the kickoff plan in `docs/sprints/ui-overhaul-2-0-plan.md` and tied it to the approved out-of-roadmap UI migration scope
  - promoted the `UIM-01` through `UIM-10` backlog in `docs/sprints/out-of-roadmap-ui-migration-github-ready.md` to live GitHub issues `#59` through `#68` under milestone `Out-Of-Roadmap UI Migration`
- Validation Plan:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate Focus:
  - execute the shell-first sequencing with the live backlog: `#59` -> `#61` -> `#62` -> `#60` -> `#63`

## 2026-04-19 - Sprint 2B Settled-Surface Session Contract (#52-#54)

- Completed:
  - added `GET /api/auth/session` and moved the native shell to a cookie-backed browser-session bootstrap so `/app/login` and `/app` render through the settled SSR shell without exposing demo credentials on the public sign-in surface
  - updated native login success handling to redirect through the intended shell contract instead of relying on a JavaScript bearer bootstrap roundtrip
  - shifted touched `forms` and `hierarchy` handlers onto `AuthenticatedRequest` and gated the settled `Forms`, `Organization`, and `Workflows` hydrate loaders on confirmed authenticated session state to avoid pre-session fetch races
  - tightened the end-to-end auth/session suite so browser tests sign in through the cookie session contract, scripted token helpers stay isolated from browser sessions, and response/workflow route assertions target the hardened SSR behavior
  - refreshed `scripts/smoke.ps1` and `scripts/uat-sprint.ps1` to validate the Sprint 2B auth/session contract against the rebuilt local app instead of the old bearer-driven expectations
- Validation:
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-web`
  - `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-web --no-default-features --features hydrate`
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Notes:
  - the full `cargo test -p tessara-api` command remains unusually slow in this environment, but the smoke matrix and touched API integration suites passed against the rebuilt Sprint 2B stack
  - the workflow assignment-console regression exposed one more session-bootstrap race on `Workflows`; that loader is now gated the same way as `Forms` and `Organization`
- Next:
  - close the Sprint 2B issue set in GitHub and prepare the remaining sprint-closeout documentation once reviewer verification is complete

## 2026-04-18 - Sprint 2B Auth Foundation (#46-#51)

- Completed:
  - added `018_auth_session_hardening.sql` to migrate `account_credentials` from raw passwords to Argon2-backed storage and to extend sessions with expiry, revocation, and last-seen tracking columns
  - replaced the monolithic API auth module with bounded auth/session modules under `crates/tessara-api/src/auth/` covering DTOs, repository access, service logic, extractors, and handlers
  - switched seeded/demo accounts and user create/edit flows to Argon2id password-hash persistence and added automatic legacy-password backfill during database startup
  - moved browser `/app` authentication to same-origin cookie sessions by removing JavaScript bearer-token bootstrap from the native runtime and retained bridge compatibility fetch paths
  - introduced `AuthenticatedRequest` and rolled it into the touched auth-facing handlers in `auth`, `users`, `demo`, and `app_summary`
  - replaced raw auth/session error exposure with stable application error envelopes and traced server-side logging
  - fixed demo seed runtime linkage so seeded submissions populate workflow runtime foreign keys and auth/session regression tests observe the true workflow state
- Validation:
  - `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-api`
  - `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-web --no-default-features --features hydrate`
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-api --test workflow_runtime -- --nocapture --test-threads=1`
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-api --test demo_flow user_management_supports_create_edit_and_account_status -- --nocapture`
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-web`
- Notes:
  - issue `#50` is complete for the touched auth/session handlers in this slice, but untargeted compatibility handlers still use older helper wrappers until later backend decomposition work
  - the browser contract is now cookie-first for `/app`, while bearer tokens remain available for explicit test/script flows and API-oriented request helpers
- Next:
  - begin issue [#52](https://github.com/ericwburden/tessara/issues/52) for native SSR ownership of `/app/login` and `/app`
  - follow with issue [#53](https://github.com/ericwburden/tessara/issues/53) for settled `Organization` and `Forms` route ownership under the hardened auth/session contract

## 2026-04-18 - Sprint 2B Kickoff

- Completed:
  - confirmed `main` was clean and aligned with `origin/main` before sprint kickoff
  - verified `Sprint 2B: Authentication Hardening And Settled-Surface Native SSR Slice` is the sole `(Next)` marker in `docs/roadmap.md`
  - created the Sprint 2B worktree at `D:\Projects\tessara-sprint-2b` on branch `codex/sprint-2b`
  - added the kickoff plan in `docs/sprints/sprint-2b-plan.md`
  - assessed the live Sprint 2B GitHub issue chain and confirmed [#46](https://github.com/ericwburden/tessara/issues/46) through [#54](https://github.com/ericwburden/tessara/issues/54) all remain open
- Assessment:
  - the sprint is serialized through the auth/session foundation: `2B-01` -> `2B-02` -> `2B-03`
  - `2B-04` depends on `2B-03`
  - `2B-05` depends on `2B-02` and `2B-03`
  - `2B-06` depends on `2B-02` through `2B-05`
  - the first useful parallel frontend split is `2B-07` and `2B-08`, both of which depend on `2B-03`, `2B-05`, and `2B-06`
  - `2B-09` remains the sprint closeout gate on top of the full stack
- Next:
  - begin implementation with [#46](https://github.com/ericwburden/tessara/issues/46) for password-hash schema and migration/backfill
  - treat [#47](https://github.com/ericwburden/tessara/issues/47) and [#48](https://github.com/ericwburden/tessara/issues/48) as the immediate follow-on foundation issues before opening route migration work

## 2026-04-17 - Sprint 2A Final Closeout Verification

- Completed:
  - reran the full Sprint 2A closeout verification sequence after the final response-queue, role-aware navigation, and delegate-context repaint fixes landed
  - refreshed stale closeout automation to match the native SSR contract on the touched `Home`, `Forms`, `Workflows`, and `Responses` surfaces in `scripts/uat-sprint.ps1` and `scripts/smoke.ps1`
  - updated the end-to-end suite so admin-only native-shell assertions sign in before checking admin navigation, and filtered the known benign wasm-download-abort message that occurs when Playwright replaces a page during navigation before the previous wasm stream finishes loading
  - completed manual Sprint 2A UAT and recorded the final pass state in `docs/context/sprint-2a-manual-uat-2026-04-17.md`
- Validation:
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `.\scripts\smoke.ps1`
  - `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-web --no-default-features --features hydrate`
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-api`
  - `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-web`
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`
  - `C:\Users\ericw\.cargo\bin\cargo.exe fmt --all`
- Notes:
  - local Playwright execution still uses the repository Node entrypoint because `npx` is not available on this PowerShell `PATH`
  - the Playwright console guard continues to fail on real console errors; it now ignores only the page-replacement wasm abort message that is emitted when the browser cancels a previous document load during navigation
- Next Sprint: Sprint 2B Draft, Submit, And Review Response Slice

## 2026-04-17 - Sprint 2A Workflow Assignment And Response Start Closeout

- Completed:
  - finished the Sprint 2A touched-surface migration by replacing the remaining transitional `Forms` create/edit routes with native Leptos-owned SSR pages and native hydrate-side form authoring behavior
  - fixed true client hydration for the native shell by restoring hydration startup, configuring the wasm executor, and removing server/client DOM divergence from the shared `NativePage` shell
  - made the touched-shell navigation, theme controls, responsive sidebar behavior, and forms/workflows/responses route flows pass native-shell browser coverage without `app-legacy.js`
  - updated the local launch path for the installed Docker Compose CLI by removing the unsupported `down --remove-orphans` flag from `scripts/local-launch.ps1`
  - expanded Sprint 2A browser coverage so forms authoring routes and left-nav route changes are explicitly guarded as native SSR behavior
- Validation:
  - `cargo fmt --all`
  - `cargo check -p tessara-web --no-default-features --features hydrate`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`
  - local Playwright required a portable Node runtime because `npx`/`npm` were not available on this PowerShell `PATH`; the suite itself passed unchanged
- Next Sprint: Sprint 2B Draft, Submit, And Review Response Slice

## Sprint Handoff / Demo Instructions

### Native SSR Product Shell
- Role: admin
- Paths:
  - `http://localhost:8080/app`
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/workflows`
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app` and confirm the shared shell loads without stale content or console errors.
  3. Use the left nav to move through `Forms`, `Workflows`, and `Responses`.
  4. Use browser back/forward once across those routes.
- Expected:
  - URL and visible content change together, the shell stays responsive, and touched routes no longer depend on the hybrid bridge script
- Acceptance check:
  - Sprint 2A touched routes behave as one native SSR shell with successful hydration rather than remount fallback
- Evidence location:
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`
  - `.\scripts\local-launch.ps1`

### Workflow Assignment And Start
- Role: admin, respondent
- Paths:
  - `http://localhost:8080/app/workflows`
  - `http://localhost:8080/app/workflows/assignments`
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as `admin@tessara.local` and open `/app/workflows`.
  2. Open a workflow, then open the shared assignment console and verify assignments exist and can be reviewed.
  3. Sign in as `respondent@tessara.local` and open `/app/responses`.
  4. Start pending work and confirm the item moves into the draft response flow.
- Expected:
  - workflow assignments are managed from the workflow/runtime UI and pending work launches the native response editor without builder tooling
- Acceptance check:
  - assigned users can discover and start only assignment-backed work through the intended application surfaces
- Evidence location:
  - `crates/tessara-api/tests/workflow_runtime.rs`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

### Forms Authoring On Native SSR
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/new`
  - `http://localhost:8080/app/forms`
- Steps:
  1. Open `/app/forms/new` and confirm the route loads natively with metadata controls and no bridge script.
  2. Return to `/app/forms`, open an existing form edit route, and confirm the draft workspace loads.
  3. Verify the page stays interactive after hydration and the shell behaves correctly on narrow widths.
- Expected:
  - full `/app/forms*` coverage for list, create, detail, and edit now lives on the native SSR platform
- Acceptance check:
  - Sprint 2A no longer leaves a touched `Forms` authoring route on the transitional shell
- Evidence location:
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`
  - `cargo test -p tessara-web`

### Delegate-Aware Response Queue
- Role: delegator
- Paths:
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as `delegator@tessara.local`.
  2. Open `/app/responses` and switch to delegated context.
  3. Confirm pending, draft, and submitted work reflect the delegate view through the same native response surface.
- Expected:
  - delegate-context response access still works after the workflow runtime and native shell migration
- Acceptance check:
  - Sprint 2A runtime changes preserve delegated response work discovery and review
- Evidence location:
  - `cargo test -p tessara-api`
  - `.\scripts\smoke.ps1 -KeepServices`

## Acceptance Mapping

- Exit condition:
  - a tester can assign work and start the correct response flow without builder tooling, while the runtime foundation remains ready for later multi-step expansion
- Manual demonstration:
  - `Workflow Assignment And Start`
  - `Delegate-Aware Response Queue`
- Automated check:
  - `cargo test -p tessara-api`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

- Exit condition:
  - Sprint 2A-touched `Home`, `Forms`, `Workflows`, and `Responses` surfaces are off the hybrid shell and operating as native SSR routes with successful hydration
- Manual demonstration:
  - `Native SSR Product Shell`
  - `Forms Authoring On Native SSR`
- Automated check:
  - `cargo check -p tessara-web --no-default-features --features hydrate`
  - `cargo test -p tessara-web`
  - `cd end2end; node .\node_modules\@playwright\test\cli.js test`

## 2026-04-16 - Sprint Delivery Rule Update For Hybrid Shell Removal

- Decision:
  - every sprint must collapse the hybrid shell on any route or UI surface it touches
  - a sprint is not complete if its touched surfaces still rely on `application.rs` HTML-string shells, `inner_html` route injection, or `app-legacy.js`
  - the roadmap end-state now explicitly requires the hybrid shell to be completely removed rather than merely contained
- Roadmap impact:
  - Sprint 2A is reopened and remains in progress until the Sprint 2A-touched `Home`, `Forms`, `Workflows`, and `Responses` surfaces are fully migrated to native SSR ownership
  - Sprint 2B stays queued behind Sprint 2A closeout rather than becoming active immediately
- Remaining Sprint 2A closeout work under the new rule:
  - replace the transitional `extract_app_root(...)+inner_html` pattern on touched surfaces with native Leptos-rendered layout and route components
  - remove `app-legacy.js` dependency from the touched Sprint 2A product surfaces
  - move touched-surface navigation to router-native SSR behavior instead of the current full-page fallback navigation
  - extend end-to-end coverage so touched-route navigation and browser-console health are part of closeout validation

## 2026-04-16 - Sprint 2A Workflow Assignment And Response Start Closeout

- Completed:
  - landed the Sprint 2A workflow runtime foundation with first-class `workflows`, `workflow_versions`, `workflow_steps`, `workflow_assignments`, `workflow_instances`, and `workflow_step_instances`
  - linked submissions to workflow runtime records without physically renaming the existing submission storage
  - added workflow CRUD, publish, assignment, pending-work, and response-start APIs in `crates/tessara-api/src/workflows.rs`
  - replaced the bridge-driven Responses route behavior with native route ownership and added top-level `Workflows` product routes in `crates/tessara-web`
  - updated form detail workflow cross-linking and workflow/runtime compatibility behavior in demo seeding and legacy import paths
  - fixed the fresh-database migration backfill query shape in `crates/tessara-api/migrations/017_workflow_runtime.sql`
  - fixed the hydrate-side web transport helpers and workflow-page event wiring so the `cargo-leptos` production build succeeds
  - fixed assignment-backed response start compatibility so respondent and delegate response options still surface seeded assignment work after the workflow cutover
- Validation:
  - `cargo check -p tessara-web --no-default-features --features hydrate`
  - `.\scripts\local-launch.ps1`
  - `cargo test -p tessara-api --test demo_flow role_based_access_respects_scope_and_respondent_context -- --exact --nocapture --test-threads=1`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `cd end2end; npx playwright test`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Roadmap position:
  - Sprint 2A functional scope is implemented, but the sprint is reopened by the touched-surface SSR migration rule
  - Sprint 2B is not active until Sprint 2A closes under that rule

## 2026-04-15 - Sprint 2A Workflow Runtime Pickup Note

- Roadmap position:
  - Sprint 1G is complete
  - Sprint 2A is now in progress rather than merely next
- Completed in the interrupted Sprint 2A worktree:
  - added workflow runtime schema, backfill migration, and workflow-aware submission linkage
  - added workflow CRUD, publish, assignment, pending-work, and response-start API surfaces
  - added top-level `/app/workflows*` routes and native response/workflow pages in `crates/tessara-web`
  - updated forms detail cross-linking and smoke coverage for workflow shells and pending work
  - fixed the fresh-database migration failure in `017_workflow_runtime.sql` so full `cargo test -p tessara-api` now passes
- Verified so far:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
- Still open before Sprint 2A closeout:
  - rerun `.\scripts\smoke.ps1` to completion after the new Postgres retry helper change
  - run `.\scripts\local-launch.ps1` to a healthy seeded instance; the prior attempt was interrupted mid-execution
  - run `cd end2end; npx playwright test` against that live local instance; root-level `npx playwright test` is not the correct invocation for this repo
  - finish manual UAT for workflow assignment, pending work, and native response start
- Pickup reference:
  - `D:\Projects\tessara\docs\context\sprint-2a-workflow-runtime-pickup-2026-04-15.md`

## 2026-04-15 - Roadmap Update For Multi-Step Workflows

- Decision:
  - keep Sprint 2A bounded to assignment and response-start on top of the new workflow runtime foundation
  - add explicit roadmap coverage for multi-step workflow authoring and runtime progression instead of leaving it as an implied future enhancement
- Completed:
  - updated `D:\Projects\tessara\docs\roadmap.md` so Phase 2 now includes `Sprint 2C: Multi-Step Workflow Authoring And Execution`
  - moved runtime-status and materialization work to `Sprint 2D` so multi-step workflow support is scheduled before downstream runtime-monitoring refinement
- Roadmap impact:
  - Sprint 2A remains the active sprint
  - multi-step workflows are now a committed roadmap deliverable rather than a deferred note with no slot

## 2026-04-15 - Sprint 1G Tessara UI Component System Foundation Closeout

- Completed:
  - restored `crates/tessara-ui` as a real workspace crate and routed shared application markup through reusable primitives for page headers, action groups, cards, panels, metadata strips, fields, and toolbars
  - migrated the shared route scaffolding in `crates/tessara-web/src/application.rs` so home, directory, detail, and editor surfaces now share the same component layer instead of accumulating route-local markup drift
  - moved common authoring and filter surfaces for forms, reports, dashboards, users, roles, node types, and response-start onto shared field and toolbar wrappers
  - added shared primitive-contract guidance that is now consolidated into `docs/ui-guidance.md`
  - fixed the analytics projection in `crates/tessara-api/src/analytics.rs` so unlabeled draft form versions no longer break demo seeding or local launch
- Validation:
  - `cargo fmt --all`
  - `cargo test -p tessara-ui`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `cd end2end; npx playwright test`
- Next Sprint: Sprint 2A Workflow Assignment And Response Start

## Sprint Handoff / Demo Instructions

### Shared Component Shell
- Role: admin
- Paths:
  - `http://localhost:8080/app`
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/responses`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app` and confirm the header shows grouped actions plus a compact metadata strip below the page title.
  3. Open `/app/forms` and `/app/responses`.
  4. Confirm the route sections use the same card, panel, and action treatment rather than route-specific button and header variants.
- Expected:
  - shared routes present one recognizable component system with consistent headers, metadata strips, action groups, cards, and panels
- Acceptance check:
  - a reviewer can move between home, forms, and responses without encountering visibly different page-shell patterns for the same UI jobs
- Evidence location:
  - `crates/tessara-ui/src/lib.rs`
  - `crates/tessara-web/src/application.rs`
  - `cd end2end; npx playwright test`

### Shared Authoring Controls
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/new`
  - `http://localhost:8080/app/reports/new`
  - `http://localhost:8080/app/administration/users/new`
  - `http://localhost:8080/app/administration/node-types/new`
- Steps:
  1. Open each route from the shared shell.
  2. Confirm labels, controls, helper text, and action rows share one field-wrapper treatment.
  3. On report, role, user-access, and node-type flows, confirm dense filter/action areas render inside compact toolbar or panel-header containers rather than ad hoc inline markup.
- Expected:
  - create and edit routes share a consistent field family and compact toolbar treatment that can be reused by Sprint 2A assignment and response-start work
- Acceptance check:
  - engineers can add another authoring or filter surface without inventing a new local markup pattern for labels, controls, or action rows
- Evidence location:
  - `crates/tessara-web/src/application.rs`
  - primitive-contract guidance now consolidated into `docs/ui-guidance.md`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

### Role-Gated Validation
- Role: operator
- Paths:
  - `http://localhost:8080/app`
  - `http://localhost:8080/app/organization`
  - `http://localhost:8080/app/forms`
- Steps:
  1. Sign in as `operator@tessara.local`.
  2. Open the shared home, organization, and forms routes.
  3. Confirm the shared component shell remains readable for non-admin work while administration-only node-type management stays denied by scripted validation.
- Expected:
  - product routes remain usable for non-admin roles while admin-only management APIs stay blocked
- Acceptance check:
  - the new shared component layer does not collapse role boundaries while it standardizes surface presentation
- Evidence location:
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Acceptance Mapping

- Exit condition:
  - a tester can move through the current shared application surfaces and see consistent headers, actions, cards, panels, and common control styling, and engineers can extend the same component layer for the next workflow-runtime sprint without inventing a new surface pattern each time
- Manual demonstration:
  - `Shared Component Shell`
  - `Shared Authoring Controls`
  - `Role-Gated Validation`
- Automated check:
  - `cargo test -p tessara-ui`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `cd end2end; npx playwright test`

## 2026-04-15 - Tessara UI Component System Detour

- Decision:
  - keep the current `cargo-leptos` plus Sass pipeline and start an internal `tessara-ui` component library instead of switching CSS frameworks or continuing to hand-produce every route surface
  - use the consolidated `docs/ui-guidance.md` as the component specification source for appearance and behavior
  - if a component cannot be specified confidently from those sources, stop and resolve the spec gap before adding a new bespoke pattern
- Completed:
  - created `crates/tessara-ui` and added it to the workspace
  - extracted the first shared primitives: buttons, action groups, page headers, metadata strips, cards, and panels
  - refactored initial application surfaces to consume those primitives so the component layer is real rather than only planned
- Planned next work:
  - continue the explicit `Sprint 1G` detour on top of `tessara-ui` rather than route-local markup
  - extract the next component families needed for workflow assignment and response-start work: inputs, field wrappers, section layouts, and table or list toolbar primitives
  - keep the shared shell stable while moving new and touched surfaces onto the component layer incrementally
- Validation:
  - `cargo fmt --all`
  - `cargo test -p tessara-ui`
  - `cargo test -p tessara-web`
  - `cd end2end; npx playwright test`
- Roadmap impact:
  - inserted `Sprint 1G: Tessara UI Component System Foundation` between Sprint 1F and Sprint 2A
  - Sprint 1G is now the next sprint
  - Sprint 2A no longer carries the component-system extraction as hidden scope

## 2026-04-15 - Sprint 1F Application UI Guidance Alignment Closeout

- Completed:
  - replaced the hero-first shared application chrome with a utility top app bar, static global search, left-sidebar navigation, page-local headers, and selective breadcrumbs
  - aligned the core `Home`, `Organization`, `Forms`, `Responses`, `Dashboards`, `Administration`, and `Migration` routes to a more consistent directory/detail/editor shell treatment
  - tightened responsive shell behavior for desktop, tablet, and mobile, including sidebar collapse and overlay navigation behavior without losing SSR readability
  - updated shared shell copy so product routes stop reading like builder or harness surfaces while transitional reporting remains reachable but subordinate
  - extended application route coverage in `crates/tessara-web/src/lib.rs`, `end2end/tests/app.spec.ts`, and `scripts/smoke.ps1` to guard the Sprint 1F shell contract
- Validation:
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `.\scripts\smoke.ps1 -KeepServices`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api`
  - `cd end2end; npx playwright test`
  - `cargo fmt --all`
  - refreshed the local Docker image with a no-cache rebuild before the final Playwright pass so the served app matched the Sprint 1F worktree sources
- Next Sprint: Sprint 1G Tessara UI Component System Foundation

## Sprint Handoff / Demo Instructions

### Shared Shell And Product Navigation
- Role: admin
- Paths:
  - `http://localhost:8080/app`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app`.
  3. Confirm the top bar stays visible with the Tessara mark, static search field, and theme controls.
  4. Confirm the left sidebar emphasizes `Home`, `Organization`, `Forms`, `Responses`, and `Dashboards`, with `Reports` shown under transitional analytics.
- Expected:
  - the application home renders inside the shared shell with a utility-only top bar and no hero-style shell header
  - primary product destinations are visible in the sidebar and transitional reporting is visually subordinate
- Acceptance check:
  - A reviewer can identify the shared product shell immediately and move into core product areas without landing in builder-style framing.
- Evidence location:
  - `D:\Projects\tessara-sprint-1f\tmp\ui-guidance\app-home-desktop.png`
  - `crates/tessara-web/src/application.rs`
  - `end2end/tests/app.spec.ts`

### Responsive Shell Behavior
- Role: admin
- Paths:
  - `http://localhost:8080/app`
- Steps:
  1. Open `/app` on desktop width and verify the sidebar is expanded in the two-region shell.
  2. Resize to tablet width and confirm the sidebar can collapse while the page content remains readable.
  3. Resize to mobile width and confirm the navigation opens as an overlay and closes when dismissed.
- Expected:
  - desktop keeps an expanded sidebar
  - tablet supports collapsed navigation
  - mobile uses an overlay nav without shell-level horizontal overflow
- Acceptance check:
  - The shell stays readable and controllable at desktop and narrow widths without broken layout or stranded navigation.
- Evidence location:
  - `D:\Projects\tessara-sprint-1f\tmp\ui-guidance\app-home-mobile.png`
  - `end2end/tests/app.spec.ts`
  - closeout Playwright run

### Core Route Framing
- Role: admin
- Paths:
  - `http://localhost:8080/app/organization`
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/responses`
  - `http://localhost:8080/app/dashboards`
  - `http://localhost:8080/app/administration`
  - `http://localhost:8080/app/migration`
- Steps:
  1. Open each route from the shared shell.
  2. Confirm each route renders its heading and actions inside the main workspace rather than in a shell hero.
  3. Open a deeper route such as an organization detail or form detail page and confirm breadcrumbs only appear there.
  4. Confirm Administration and Migration remain reachable but visibly secondary to the core product areas.
- Expected:
  - list, detail, and editor pages follow the shared framing
  - breadcrumbs appear only on deeper flows
  - internal/operator areas remain subordinate without becoming separate themes
- Acceptance check:
  - A reviewer can move through the current application routes and recognize one coherent app shell rather than mixed builder-era route framing.
- Evidence location:
  - `D:\Projects\tessara-sprint-1f\tmp\ui-guidance\organization-desktop.png`
  - `D:\Projects\tessara-sprint-1f\tmp\ui-guidance\migration-desktop.png`
  - `crates/tessara-web/src/application.rs`
  - `crates/tessara-web/src/lib.rs`

### Role-Gated Internal Surfaces
- Role: operator
- Paths:
  - `http://localhost:8080/app/administration`
  - `http://localhost:8080/app/migration`
- Steps:
  1. Sign in as `operator@tessara.local`.
  2. Attempt to open `/app/administration`.
  3. Attempt to open `/app/migration`.
- Expected:
  - restricted internal surfaces do not render as usable operator pages
  - the shell remains readable while access is denied appropriately
- Acceptance check:
  - Non-admin users do not gain access to internal-only surfaces while the shared shell still behaves coherently.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `scripts\smoke.ps1`
  - `end2end/tests/app.spec.ts`

## Acceptance Mapping

- Exit condition:
  - a tester can sign in and move through the existing application routes in a coherent shell on desktop and narrow widths, without builder-centric framing, shell-level horizontal scroll, hydration regressions, or browser-console errors
- Manual demonstration:
  - `Shared Shell And Product Navigation`
  - `Responsive Shell Behavior`
  - `Core Route Framing`
- Automated check:
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api`
  - `npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `.\scripts\local-launch.ps1`
  - `cargo fmt --all`

## 2026-04-14 - Sprint 1F Application UI Guidance Alignment Kickoff

- Kickoff status: started `Sprint 1F: Application UI Guidance Alignment` from clean `main`.
- Branch and worktree:
  - branch: `codex/sprint-1f`
  - worktree: `D:\Projects\tessara-sprint-1f`
- Plan file: `D:\Projects\tessara-sprint-1f\docs\sprints\sprint-1f-plan.md`
- Roadmap update:
  - inserted Sprint 1F after Sprint 1E
  - marked Sprint 1F as `(Next)` and pushed Sprint 2A back to the next slot
- Planned verification commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-api`
  - `cargo test -p tessara-web`
  - `cd end2end; npx playwright test`
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Immediate implementation focus:
  - replace the hero-first application chrome with a utility top bar, left-sidebar shell, selective breadcrumbs, and route-local page headers
  - tighten the existing core routes so Home, Organization, Forms, Responses, Dashboards, Administration, and Migration read as one coherent application
  - keep transitional reporting reachable without treating it as the primary navigation model

## 2026-04-14 - Form Versioning Pivot To Major-Version Compatibility

- Pivoted form versioning from compatibility-group-centric behavior to semantic versioning with major-version compatibility.
- Backend changes now:
  - assign `version_major`, `version_minor`, and `version_patch` on publish
  - derive `PATCH` / `MINOR` / `MAJOR` from the form contract delta at publish time
  - keep compatible publishes on the current major line
  - start a new major line for breaking publishes
  - freeze new direct dataset and direct report consumers to the current published form major automatically
- Application and builder surfaces now:
  - create unlabeled draft versions and defer final version assignment until publish
  - show publish previews and major-line compatibility messaging instead of manual compatibility-group entry
  - remove remaining compatibility-group controls from the active forms routes and the legacy admin builder dataset/form authoring surfaces
- Migration updates:
  - `015_form_version_semver.sql` introduces semantic-version fields and major-version binding fields for dataset/report consumers
  - `016_form_version_legacy_label_backfill.sql` backfills older non-semver labels such as `v1` and `legacy-v1`
- Validation:
  - `cargo fmt --all`: completed successfully on 2026-04-14
  - `cargo test -p tessara-api`: completed successfully on 2026-04-14
  - `cargo test -p tessara-web`: completed successfully on 2026-04-14
  - `scripts\local-launch.ps1`: completed successfully on 2026-04-14 after splitting the legacy-label backfill into migration `016`
  - `scripts\smoke.ps1`: completed successfully on 2026-04-14 against the revised major-version model

## 2026-04-14 - Sprint 1D Forms, Fields, And Version Authoring Closeout

- Completed Sprint 1D closeout work in `D:\Projects\tessara`.
- Delivered application-owned forms route coverage for:
  - `/app/forms`
  - `/app/forms/new`
  - `/app/forms/{form_id}`
  - `/app/forms/{form_id}/edit`
- Delivered form-authoring workflow updates in the application shell:
  - top-level form create/edit flows continue through native app routes instead of builder-only fallback behavior
  - form detail now surfaces version summary, workflow attachments, and section/field preview panels
  - form edit now supports draft version creation, section add/update/delete/reorder, field add/update/delete/reorder, and draft publish actions
  - publish validation and stale/double-submit protection are surfaced in route-local status messages
  - option-set and lookup touchpoints remain visible as non-blocking read-only affordances where backend metadata is not yet available
- Expanded closeout evidence coverage for the forms slice:
  - `scripts\uat-sprint.ps1` now checks forms list, new, detail, and edit routes
  - `scripts\smoke.ps1` now checks forms lifecycle and authoring route markers
  - `end2end\tests\app.spec.ts` now includes forms route render and JS-disabled readability checks
- Roadmap update:
  - Marked Sprint 1D as complete in `D:\Projects\tessara\docs\roadmap.md`.
  - Marked Sprint 2A as the next sprint focus.
- Validation:
  - `scripts\local-launch.ps1`: completed successfully on 2026-04-14.
  - `scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`: completed successfully on 2026-04-14.
  - `scripts\smoke.ps1`: completed successfully on 2026-04-14 after updating stale `demo_flow` assertion expectations to match current API behavior.
  - `cargo fmt --all`: completed successfully on 2026-04-14.
  - `cargo test -p tessara-api`: completed successfully on 2026-04-14.
  - `cargo test -p tessara-web`: completed successfully on 2026-04-14.
- Next Sprint: Sprint 2A Workflow Assignment And Response Start

## Sprint Handoff / Demo Instructions

### Forms Directory And Lifecycle Visibility
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/api/forms`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app/forms`.
  3. Verify the forms directory renders cards with scope, published version, and draft-count summaries.
  4. Open one form detail route from the list.
- Expected:
  - the forms list renders without builder-only fallback navigation
  - lifecycle information is visible before entering detail
- Acceptance check:
  - Admin can browse the forms directory and identify published and draft state from the route itself.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `scripts\smoke.ps1`
  - `end2end\tests\app.spec.ts`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Form Creation And Native Route Ownership
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/new`
  - `http://localhost:8080/api/admin/forms`
- Steps:
  1. Open `/app/forms/new`.
  2. Enter a form name, slug, and optional scope node type.
  3. Submit the form.
  4. Confirm the browser redirects into `/app/forms/{form_id}/edit`.
- Expected:
  - the form can be created from the application route
  - the route continues directly into version authoring
- Acceptance check:
  - Admin can create a form without using the legacy builder and land in the authoring route.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Version Authoring, Sections, Fields, And Publish
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/{form_id}/edit`
  - `http://localhost:8080/api/admin/forms/{form_id}/versions`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/sections`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/fields`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/publish`
- Steps:
  1. Open `/app/forms/{form_id}/edit`.
  2. Create a draft version.
  3. Add a section.
  4. Add one or more fields to that section.
  5. Reorder a section and a field.
  6. Publish the draft version.
- Expected:
  - draft lifecycle controls are visible in the route
  - section and field authoring actions are available without leaving the app route
  - invalid publish attempts show explicit route-local validation messages
- Acceptance check:
  - Admin can complete create draft version -> add section -> add field -> reorder -> publish entirely through `/app/forms/{id}/edit`.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `scripts\smoke.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Form Detail Review And Workflow Attachments
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/{form_id}`
  - `http://localhost:8080/api/forms/{form_id}`
  - `http://localhost:8080/api/form-versions/{form_version_id}/render`
- Steps:
  1. Open `/app/forms/{form_id}`.
  2. Verify the summary section shows scope and published/draft state.
  3. Verify the version summary panel renders semantic version, major-line compatibility, and publish metadata.
  4. Verify section and field preview panels render.
  5. Verify related reports and dataset-source workflow attachments are visible.
- Expected:
  - detail route clearly separates summary, version lifecycle, section preview, and workflow attachments
  - route remains readable without JavaScript for core headings and structure
- Acceptance check:
  - A tester can inspect version status and downstream workflow links entirely from the form detail route.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `end2end\tests\app.spec.ts`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Access Control And Non-Admin Readability
- Role: operator
- Paths:
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/forms/{form_id}`
  - `http://localhost:8080/app/forms/{form_id}/edit`
  - `http://localhost:8080/api/forms`
  - `http://localhost:8080/api/admin/forms/{form_id}`
- Steps:
  1. Sign in as `operator@tessara.local`.
  2. Open `/app/forms` and a visible form detail route.
  3. Attempt to open the edit route or call an admin forms endpoint.
  4. Confirm readable forms surfaces remain available while write/admin actions stay gated.
- Expected:
  - readable routes remain usable where the role has access
  - admin-only write flow remains restricted
- Acceptance check:
  - At least one allowed read path and one denied write/admin path are both demonstrated.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

## Acceptance Mapping

- Exit condition:
  - a tester can create a form, add/edit/remove/reorder fields, publish a version, and inspect status entirely through UI
- Manual demonstration:
  - `Form Creation And Native Route Ownership`
  - `Version Authoring, Sections, Fields, And Publish`
  - `Form Detail Review And Workflow Attachments`
- Automated check:
  - `scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `scripts\smoke.ps1`
  - `end2end\tests\app.spec.ts`

## 2026-04-13 - Sprint 1C Organization Management Closeout

- Completed organization-management closure work in `D:\Projects\dms-migration\tessara`.
- Added organization scope-aware hierarchy browsing and editing in native application routes:
  - `/app/organization` now uses full-tree directory navigation and destination labelling based on scoped node types.
  - `/app/organization/{node_id}` now uses tree-aware detail framing with path, metadata, and add-child actions derived from configured node-type relationships.
  - `/app/organization/new` and `/app/organization/{node_id}/edit` now initialize from node-type metadata and hierarchy rules.
- Added complete organization node-type admin flow in application shell:
  - `/app/administration/node-types` list
  - `/app/administration/node-types/new`
  - `/app/administration/node-types/{node_type_id}`
  - `/app/administration/node-types/{node_type_id}/edit`
- Backend updates in `tessara-api` for the same slice:
  - schema migration `014_node_type_labels.sql` adds singular/plural node-type labeling fields
  - node-type catalog now exposes readable labels and parent/child relationship graph (`/api/node-types`)
  - node-type CRUD enforces relationship consistency, cycle avoidance, access control (`admin:all`), and non-root parent requirements for non-root types
  - node metadata field deletion support (`DELETE /api/admin/node-metadata-fields/{field_id}`)
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo test -p tessara-api --test demo_flow readable_node_type_catalog_exposes_labels_and_relationships`
  - `cargo test -p tessara-api --test demo_flow node_metadata_fields_can_be_deleted`
  - `cargo test -p tessara-api --test demo_flow non_root_node_types_require_a_parent_node`
  - `cargo test -p tessara-api --test demo_flow operator_cannot_access_admin_node_type_management_routes`
  - `cargo test -p tessara-api --test demo_flow node_type_updates_reject_cycles_in_parent_child_selections`
  - `cargo test -p tessara-web`
- Roadmap update:
- Marked Sprint 1C as complete in `D:\Projects\dms-migration\tessara\docs\roadmap.md`.
  - Marked Sprint 1D as the next sprint focus.

## 2026-04-08

- Added progress-report tracking in the docs root at `D:\Projects\dms-migration\tessara\docs\progress-report.md`.
- Current roadmap position:
  - Slices 11-13 implemented.
  - Slice 14 implemented in a limited v1 form.
  - Slice 15 largely implemented.
  - Slice 16 partially implemented.
  - Slice 17 partially implemented.
  - Slices 18-23 remain.
- Latest completed implementation milestones in `tessara`:
  - `1b55038 Expose source-aware dataset report previews`
  - `e5ffd6a Model dataset composition modes`
  - `1aa6fd1 Add avg min and max aggregation metrics`
- Roadmap was updated to make the next major phase explicit:
  - Slice 18: Real Application Shell and Navigation
  - Slice 19: Entity Lists, Detail Views, and Creation Menus
  - Slice 20: Submission, Admin, and Reporting Workflow Parity
- Next planned development focus:
  - begin the real application shell
  - add home screen and persistent navigation
  - start replacing workbench-style routes with original-project-inspired application structure

## 2026-04-08

- Completed the first Slice 18 implementation checkpoint for the real application UI.
- Added a real application home at `/app` with:
  - overview content
  - persistent navigation
  - create-menu entry points
  - quick-start actions
- Split the submission workflow onto `/app/submissions` so the home route can act as a true landing page.
- Kept `/app/admin`, `/app/reports`, and `/app/migration`, but moved them under the shared application-frame structure with:
  - consistent navigation
  - shared create menu
  - shared output panels
- Updated smoke coverage to verify:
  - `/app` home shell
  - `/app/submissions`
  - `/app/admin`
  - `/app/reports`
  - `/app/migration`
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Next planned development focus:
  - continue Slice 18-20 by turning the new shell into real list/detail and creation-menu flows
  - begin replacing utility-style entry points with app-style entity screens
  - continue Slice 16 join execution in parallel with the new UI shell work

## 2026-04-08

- Completed the next Slice 18 checkpoint for the real application UI.
- Expanded `/app/admin` from a single setup screen into a clearer management workspace with:
  - `Management Areas` entry cards for hierarchy, forms, reporting, and dashboards
  - an `Entity Directory` for node types, nodes, forms, datasets, reports, aggregations, charts, and dashboards
  - direct screen-opening and data-loading actions tied to the existing admin APIs
- Updated the Leptos admin-shell tests and smoke checks so the new management structure is part of the regular quality gate.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 is in progress with the application home, persistent navigation, and the first admin management-area shell.
  - Slice 19 is next: stronger entity lists, detail views, and creation-menu flows.
  - Slice 16 dataset join execution still remains parallel backend work.
- Next planned development focus:
  - add app-style list/detail entry points for reporting and dataset management
  - continue replacing raw utility flows with screen-specific application interactions
  - begin the next set of entity-oriented routes inside the new application shell

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for reporting.
- Expanded `/app/reports` into a clearer reporting landing area with:
  - `Reporting Areas` cards for datasets, reports, aggregations, and dashboards
  - a `Reporting Directory` for dataset, report, aggregation, chart, and dashboard list entry points
  - direct loading actions tied to the existing reporting APIs and preview screens
- Kept the existing report runner and dashboard preview screens underneath this new route-level structure so reporting stays testable while the UI becomes more application-like.
- Updated the Leptos route tests and smoke checks so the reporting landing structure is part of the normal validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in progress with route-level home/navigation structure now in place for home, admin, and reporting.
  - Slice 19 is actively underway through route-level entity directories and management-area entry points.
  - Slice 16 dataset join execution is still pending as parallel backend work.
- Next planned development focus:
  - continue route-level application-shell upgrades for migration and submission contexts
  - add stronger list/detail entry points so entity browsing relies less on raw IDs
  - return to dataset join execution after the next UI-shell checkpoint

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for migration.
- Expanded `/app/migration` into a clearer operator route with:
  - `Migration Stages` cards for fixture intake, validation, dry run, and import
  - a `Migration Directory` for fixture examples, validation, dry runs, and imports
  - direct actions wired to the existing legacy-fixture APIs
- Kept the existing validation and import workbench surfaces underneath this route so migration rehearsal remains testable while the UI becomes more structured.
- Updated the Leptos route tests and smoke checks so the new migration landing structure is part of the regular validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in progress, but the focused routes now have route-level structure for home, admin, reporting, and migration.
  - Slice 19 continues through stronger entity directories and route entry points.
  - Slice 16 dataset join execution remains outstanding backend work.
- Next planned development focus:
  - improve the submissions route so it behaves more like a real list/detail application area
  - keep reducing raw-ID dependence across entity browsing
  - return to dataset join execution once the next UI-shell checkpoint lands

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for submissions.
- Expanded `/app/submissions` into a clearer route-level workspace with:
  - `Submission Stages` cards for response entry, target selection, response review, and related reports
  - a `Response Directory` for published forms, target nodes, draft responses, submitted responses, all responses, and related reports
  - direct actions wired to the existing published-form, node, submission, and report APIs
- Kept the detailed submission, review, and related-report screens underneath this route so the application shell gets more structured without losing the currently testable workflows.
- Updated the Leptos route tests and smoke checks so the new submissions landing structure is part of the regular validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 route-level application shell work is now in place across home, submissions, admin, reporting, and migration.
  - Slice 19 continues through stronger list/detail entry points and reduced raw-ID dependence.
  - Slice 16 dataset join execution remains the next major backend gap.
- Next planned development focus:
  - move back to dataset join execution with explicit join semantics and diagnostics
  - continue refining list/detail entry points where the application shell still depends too heavily on raw IDs

## 2026-04-08

- Completed the next Slice 16 backend checkpoint for dataset composition.
- Added execution support for join-mode dataset tables when:
  - the dataset uses submission grain
  - the dataset has at least two sources
  - every source uses `latest` or `earliest` selection so each source resolves to one row per node
- Join-mode dataset execution now merges selected source rows by node and returns one dataset row with:
  - combined field values across sources
  - a joined `submission_id` trace showing the contributing source/submission pairs
  - `source_alias` set to `join`
- Left dataset-backed reports on union-only execution for now, so report-engine refactoring remains a later slice rather than being mixed into this backend checkpoint.
- Added DB-backed integration coverage for:
  - successful join-mode dataset execution across two forms on the same node
  - invalid join-mode execution when a source uses the `all` selection rule
  - clearer diagnostics for single-source join datasets
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 route-level application shell work is in place across all focused app routes.
  - Slice 19 continues through stronger list/detail entry points and reduced raw-ID dependence.
  - Slice 16 has now advanced from modeled join semantics to actual join-mode dataset table execution.
- Next planned development focus:
  - continue Slice 16 by deciding how dataset-backed reports should query joined datasets
  - keep tightening application-shell list/detail flows where workbench patterns are still visible

## 2026-04-08

- Completed the next Slice 16 backend checkpoint for joined-dataset reporting.
- Added support for dataset-backed reports to run against join-mode datasets by reusing the internal dataset execution path and projecting report bindings over the joined dataset rows.
- Join-backed reports now support:
  - direct dataset field bindings
  - `literal:` computed expressions
  - `bucket_unknown` handling for missing joined values
  - joined submission traces carried through report output
- Kept the existing SQL-backed path for union datasets and form-backed reports, so this change extends the reporting model without forcing a broader report-engine rewrite in the same slice.
- Added validation/test coverage for:
  - creating and running a report on a joined dataset
  - preserving the clearer diagnostics for invalid join datasets
  - pure helper behavior for literal computed expressions and joined missing-data handling
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 has advanced from modeled joins to join-mode dataset execution and joined-dataset report execution.
  - Slice 18 route-level application shell work is in place across all focused routes.
  - Slice 19 remains active through list/detail and reduced raw-ID UI work.
- Next planned development focus:
  - decide whether charts/dashboards need explicit joined-dataset affordances next
  - continue application-shell list/detail improvements where entity browsing still feels workbench-like

## 2026-04-08

- Completed the next reporting-stack verification checkpoint for joined datasets.
- Added DB-backed integration coverage proving that:
  - a report built on a joined dataset can feed an aggregation definition
  - the aggregation engine correctly computes metrics over joined-dataset report rows
  - the joined reporting path behaves like a first-class reporting source rather than a dead-end dataset preview
- This was mainly a coverage/hardening checkpoint rather than a large new code-path change, but it closes an important uncertainty in the dataset-first reporting roadmap.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
- Current roadmap position:
  - Slice 16 now covers join-mode dataset execution, joined-dataset reports, and aggregation consumption of joined report rows.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 remains the next major UX area to continue.
- Next planned development focus:
  - return to list/detail and entity-browsing improvements in the application shell
  - add more app-style reporting/admin detail flows where raw-ID workflows still dominate

## 2026-04-08

- Completed the next reporting-stack hardening checkpoint for joined datasets.
- Added DB-backed integration coverage proving that a joined-dataset report can flow through:
  - an aggregation definition
  - an aggregation-backed chart
  - a dashboard component rendered from that aggregation-backed chart
- This was another verification-oriented slice rather than a broad implementation rewrite, but it materially reduces risk in the dataset-first reporting roadmap by confirming that joined data survives the full report-to-dashboard path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `cargo test --workspace`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is now covered through dataset execution, reports, aggregations, and dashboard consumption.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 remains the next major area to continue.
- Next planned development focus:
  - return to app-style list/detail improvements
  - continue reducing raw-ID-heavy workflows in admin and reporting screens

## 2026-04-08

- Completed the next Slice 19 reporting-route increment.
- Fixed a concrete usability/consistency gap in the focused reporting application route:
  - the route-level shell already advertised dataset entry points
  - the focused app controller now actually supports dataset browsing, inspection, and dataset-result preview on `/app/reports`
- Added:
  - dataset context selection in the focused reporting route
  - dataset definition inspection in the focused reporting route
  - dataset table preview in the focused reporting route
  - report-context selection that carries dataset context when a report is dataset-backed
- Updated route tests and smoke checks so the reporting route now validates:
  - `Choose Dataset`
  - `Inspect Dataset`
  - `Run Dataset`
  - focused dataset API references in the app shell
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is substantially advanced and now covered through dataset, report, aggregation, and dashboard paths.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 continues through stronger list/detail flows and reduced raw-ID dependence in focused app routes.
- Next planned development focus:
  - continue list/detail and entity-browsing improvements on the focused app routes
  - target the next workbench-heavy admin/reporting flow that still lacks good application-style selection and inspection

## 2026-04-08

- Completed a follow-on Slice 19 UI hardening increment for dataset browsing.
- Fixed a concrete joined-dataset usability issue in both the focused reporting route and the admin workbench:
  - dataset preview rows previously assumed one submission ID per row
  - joined dataset rows now render per-source submission actions instead of a broken single “Open Submission” action
- This keeps joined dataset previews usable now that join-mode datasets, joined reports, and joined-report aggregations are supported elsewhere in the stack.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is substantially advanced.
  - Slice 18 route-level application shell work is in place across all focused routes.
  - Slice 19 continues through smaller usability fixes and stronger entity/detail workflows.
- Next planned development focus:
  - continue reducing raw-ID-heavy admin/reporting flows
  - move another high-friction workbench interaction toward a more application-style detail/browse flow
## 2026-04-08

- Completed the next Slice 19 reporting-route usability increment.
- Added chart definition inspection as a first-class reporting flow instead of treating charts as ID-only launch points.
- Added a new authenticated API detail route for charts that returns:
  - chart/report or chart/aggregation linkage
  - dashboards currently using that chart
- Updated the focused reporting route so testers can:
  - inspect a chart from the chart list
  - inspect a chart from dashboard preview cards
  - follow chart detail into linked reports, aggregations, and dashboards
- Added DB-backed integration coverage for the new chart detail route on seeded report/dashboard data.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution is substantially advanced and usable through datasets, reports, aggregations, and dashboards.
  - Slice 18 route-level application shell work is in place across the focused routes.
  - Slice 19 continues through stronger entity detail and browse flows in reporting/admin screens.
- Next planned development focus:
  - continue reducing raw-ID-heavy admin/reporting interactions
  - move another reporting or admin builder seam toward a clearer list/detail workflow
  - continue Slice 20 hardening where the app still behaves like a workbench instead of a replacement UI
## 2026-04-08

- Completed a follow-on Slice 19 reporting-detail increment in the same batch.
- Extended report definition responses so report detail now includes downstream dependents:
  - aggregations built from the report
  - charts built directly from the report or from those aggregations
- Updated the focused reporting route so report detail cards now support traversal into:
  - linked aggregations
  - linked charts
- This makes report inspection behave more like an application detail page and less like a detached workbench form.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through richer entity detail/traversal flows on reporting and admin routes.
- Next planned development focus:
  - continue replacing remaining raw-ID-heavy reporting/admin interactions
  - improve another browse/detail seam in admin or submission review flows
  - continue Slice 20 hardening where workflows still feel more operator-like than end-user-like
## 2026-04-08

- Completed another Slice 19 admin/reporting detail increment.
- Extended dataset definitions so dataset detail now includes linked reports that depend on the dataset.
- Updated both the focused reporting route and the admin workbench so dataset inspection can now flow directly into report context and report detail.
- This removes another workbench-style dead end: datasets now behave more like application entities with downstream navigation instead of isolated builder records.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through stronger list/detail and cross-entity traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy admin interactions
  - improve another builder/detail seam, likely around forms or hierarchy setup
  - continue Slice 20 hardening where the UI still behaves more like an operator workbench than a replacement application
## 2026-04-08

- Completed the next Slice 19 admin-detail increment around forms.
- Added a dedicated form detail API surface so forms can now be inspected as first-class entities instead of only appearing in a builder list.
- Form detail now includes:
  - versions
  - linked reports that use the form directly
  - linked dataset sources that use the form directly or a form major line
- Updated the admin shell so testers can:
  - inspect a selected form
  - move directly from form detail into versions, linked reports, and linked datasets
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through stronger admin/reporting list/detail and cross-entity traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy hierarchy or form-builder interactions
  - improve another admin/detail seam with clearer contextual navigation
  - continue Slice 20 hardening where the UI still feels more operator-like than replacement-ready
## 2026-04-08

- Completed the next Slice 19 hierarchy-detail increment.
- Added a dedicated node-type detail API surface so node types can now be inspected as first-class admin entities instead of only appearing in the hierarchy builder list.
- Node-type detail now includes:
  - allowed parent node types
  - allowed child node types
  - metadata fields
  - forms scoped to that node type
- Updated the admin shell so testers can:
  - inspect a node type
  - move directly from node-type detail into parent/child relationship context, metadata-field context, and scoped forms
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through richer admin/reporting entity detail and traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy builder interactions, likely at the version-level form or node-detail layer
  - improve another admin/detail seam with contextual navigation
  - continue Slice 20 hardening where workflows still feel more operator-like than replacement-ready
## 2026-04-08

- Completed the first explicitly visible application-UI shift on the submissions route.
- Reworked `/app/submissions` so it now presents a route-level response workspace instead of only stacked utility panels.
- Added:
  - a response-console shell
  - queue-style entry cards for published forms, targets, drafts, and submitted responses
  - a guided-path panel describing the normal response flow
  - a split workspace layout that keeps the active entry/review/report sections together on the main side of the route
- This does not complete the application-UI transition, but it is the first change in this thread that should read more like a product workspace than a builder screen.
- Validation completed successfully:
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 application shell structure is in place.
  - Slice 19 is now continuing not only through entity-detail traversal, but also through visibly more application-like route layouts.
- Next planned development focus:
  - keep converting the focused routes into real workspace pages
  - reduce exposed ID-driven controls on the highest-traffic submission/admin surfaces
  - continue Slice 20 hardening as those routes become more end-user-facing
## 2026-04-08

- Paused non-UI roadmap work to focus directly on the application surface.
- Extended the visible workspace treatment beyond submissions so the focused routes now read more like destination pages:
  - `/app/submissions` already had the response console
  - `/app/admin` now has a configuration-console workspace shell
  - `/app/reports` now has an insight-console workspace shell
- Added route-level queue panels and guided-path content for admin and reporting so those routes no longer appear as only stacked builder sections.
- This is still an intermediate UI state, but it is a clearer step toward the desired application structure and away from a pure operator workbench.
- Validation completed successfully:
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - active focus is now intentionally UI-first
  - non-UI roadmap work is paused until the focused routes look more like the intended application
- Next planned development focus:
  - keep converting focused routes into real workspace pages
  - reduce or demote exposed ID-driven controls on submission/admin/reporting surfaces
  - continue reshaping the app shell toward the desired application information architecture before resuming deeper backend roadmap work
## 2026-04-08

- Continued the UI-only catch-up pass without extending beyond supported backend workflows.
- Reworked the remaining focused admin screens so they now use the same task/context layout already introduced on submission and reporting routes:
  - hierarchy setup now separates hierarchy actions from current hierarchy context
  - form builder now separates form actions from current form context
  - report builder now separates reporting configuration actions from current reporting-builder context
- Added shared task-panel styling so the focused routes now use one visible UI grammar instead of mixing workspace shells with older utility-style slabs.
- This is still not full replacement-grade product UI, but it is a meaningful catch-up step because the main focused routes now read more like application workspaces and less like a raw control surface.
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in place as the real application shell foundation
  - Slice 19 UI catch-up is still active and is now focused on making the focused routes visually coherent and less ID-forward
  - non-UI roadmap work remains paused by request
- Next planned development focus:
  - continue demoting raw-ID-heavy controls on the highest-traffic focused routes
  - push `/app/admin` and `/app/reports` farther toward browse/detail/task workflows instead of builder-style control clusters
  - stop short of inventing UI flows the backend cannot already support
## 2026-04-08

- Paused implementation work briefly to review provenance and the legacy application templates before continuing UI catch-up.
- Reviewed:
- `D:\Projects\dms-migration\tessara\docs\provenance\System Requirements Specification.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Software Design Document.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Requirements Traceability Matrix - Sheet1.pdf`
  - legacy templates from `app/mmi/templates` in the previous application
- Key UI findings from the prior application:
  - role-specific home pages matter
  - manage-list pages and detail/drill-down flows are the core structure
  - client/parent workflows center on assigned forms and completed forms
  - reports and dashboards are top-level destinations, not hidden utilities
  - breadcrumbs, search, and contextual actions are part of the expected navigation model
- Created `D:\Projects\dms-migration\tessara\docs\archive\docs\user-interface-design.md` to define the proposed Tessara UI structure based on those materials while still allowing usability improvements over the old application.
- Current roadmap position:
  - UI-only catch-up is still the active focus
  - the next UI work should follow the design note’s structure instead of continuing ad hoc shell refinement
- Next planned development focus:
  - align the current app shell to the role/home/directory/detail structure documented in `user-interface-design.md`
  - continue using the legacy product structure as a guide without reproducing the old UI exactly
## 2026-04-08

- Created a near-term UI implementation charter that was later consolidated into `docs/ui-guidance.md`.
- The new direction translated `user-interface-design.md` into a shorter, locked set of implementation decisions before later consolidation into `ui-guidance.md`.
- Locked decisions recorded there:
  - split product areas are the primary information architecture
  - shared home first, role-aware variants later
  - Administration and Migration remain visible but scoped internal surfaces
  - terminology is configuration-driven rather than hardcoded to legacy entities
- The new document also defines:
  - the primary product areas
  - the home strategy
  - the screen-family patterns
  - the mapping from current routes to the target UI structure
  - the acceptance criteria for considering the UI catch-up direction settled
- Current roadmap position:
  - UI-only catch-up remains active
  - the next implementation work should follow the guidance later consolidated into `ui-guidance.md` rather than continue route-by-route shell refinement without a locked direction
- Next planned development focus:
  - align the current route structure and screen composition to the guidance later consolidated into `ui-guidance.md`
  - continue making the UI behave like product destinations instead of mixed control surfaces
## 2026-04-08

- Updated `D:\Projects\dms-migration\tessara\docs\roadmap.md` to reflect the current implementation state and the new UI direction.
- The roadmap now clearly separates:
  - completed foundation and reporting work
  - active carry-forward UI gaps
  - future work as discrete, non-overlapping sprints
- Added the UI-direction-driven sprint sequence at the current point in the roadmap:
  - shared shell and product-area navigation
  - shared home and role-ready entry points
  - Organization and Forms product surfaces
  - Responses product surface
  - Reports and Dashboards product surfaces
  - Administration and Migration internal surfaces
- The later reporting/migration/hardening work is now pushed into subsequent sprints instead of overlapping with the active UI catch-up phase.
- Current roadmap position:
  - backend/reporting architecture remains substantially ahead of the UI
  - UI-only catch-up remains the active implementation focus
- Next planned development focus:
  - begin Sprint 1 from the updated roadmap
  - use the guidance later consolidated into `ui-guidance.md` as the implementation charter for the next UI changes
## 2026-04-08 - Sprint 1 UI Slice: Split Product-Area Shell

Roadmap position:
- Active focus remains Sprint 1: Shared Shell and Product-Area Navigation.
- This slice moved the app from focused utility routes toward the information architecture later consolidated into `ui-guidance.md` without extending beyond backend-supported workflows.

Completed in this slice:
- Added canonical product-area routes for Home, Organization, Forms, Responses, Reports, Dashboards, Administration, and Migration.
- Split navigation into Product Areas and Internal Areas.
- Added shared breadcrumb/title shell framing across the focused application routes.
- Reframed existing supported screens under bridge surfaces for Organization, Forms, Responses, Reports, Dashboards, and Administration.
- Preserved compatibility routes (/app/submissions, /app/admin) while shifting visible language to the new IA.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\scripts\smoke.ps1

Next UI steps:
- Continue Sprint 1 by making the shared shell more consistent across product areas.
- Begin Sprint 2-style home and entry-point refinement only where backend support already exists.
- Keep reducing raw-ID-forward route language in favor of product-area browse/detail/task framing.
## 2026-04-08 - Sprint 1 UI Slice: Keep Builder Shortcuts In Internal Areas

Roadmap position:
- Sprint 1 remains active.
- This checkpoint tightened the split between product-facing routes and internal/operator routes without adding any new backend requirements.

Completed in this slice:
- Added a shared route sidebar component for the focused application shells.
- Removed create-shortcut panels from Home, Organization, Forms, Responses, Reports, Dashboards, and Migration.
- Kept creation shortcuts in Administration, where the supported configuration workflows actually live.
- Repointed the form creation shortcut to Administration so product routes do not advertise builder-first behavior.
- Updated web tests and smoke coverage to enforce that product routes no longer expose internal create shortcuts.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\scripts\smoke.ps1

Next UI steps:
- Continue Sprint 1 shell consistency work across product routes.
- Strengthen page-level title/action framing so the main routes read like one application family.
- Keep product routes focused on browse/detail/task flows and leave configuration entry points in Administration.
## 2026-04-08 - Sprint 1 UI Slice: Unify Product-Area Page Shells

Roadmap position:
- Sprint 1 remains active.
- This checkpoint improved shell consistency across the focused application routes without expanding UI scope beyond existing backend support.

Completed in this slice:
- Added a shared AppAreaShell component for route-level hero, breadcrumb, action-row, sidebar, and main content framing.
- Moved Home, Organization, Forms, Responses, Reports, Dashboards, Administration, and Migration onto the same page-shell pattern.
- Kept the product/internal area split from the prior slice while removing duplicated route-shell markup.
- Simplified the home-route smoke assertion so it validates visible navigation content instead of brittle raw href text.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1

Next UI steps:
- Continue Sprint 1 by tightening route-level page titles, action framing, and browse/detail entry patterns.
- Keep product areas aligned as one application family while leaving unsupported workflows out of the UI.
## 2026-04-08 - Sprint 1 UI Slice: Standardize Area Landing Sections

Roadmap position:
- Sprint 1 remains active.
- This checkpoint tightened the route-level UI inside the shared shell so the focused product and internal areas use a more consistent landing-page grammar.

Completed in this slice:
- Added shared landing-section helpers for titled screen sections, management cards, and directory cards.
- Moved Organization, Forms, Responses, Administration, Reports, Dashboards, and Migration home/landing screens onto the same section/card rendering pattern.
- Preserved existing route titles, actions, and supported workflows while removing duplicated markup across the route landings.
- Kept the UI limited to backend-supported browse, review, run, and configuration flows.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1

Next UI steps:
- Continue Sprint 1 with stronger route-level browse/detail framing.
- Keep reducing the visual gap between current bridge surfaces and the intended application destinations.
- Avoid adding UI breadth beyond existing backend support.
## 2026-04-08 - Sprint 1 UI Slice: Promote Full-Size Brand Mark

Roadmap position:
- Sprint 1 remains active.
- This was a focused shell polish change within the existing application-shell work.

Completed in this slice:
- Switched img.brand-mark in the shared application shell and local admin shell from the 256 asset to the full-size 1024 icon asset.
- Increased the brand mark display size and adjusted spacing so the icon reads as a primary brand element in the shell header.
- Updated web-shell tests to assert the new full-size icon asset reference.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 route-level framing work.
- Keep improving visible application-shell coherence without adding unsupported UI behavior.
## 2026-04-08 - Local Launch Helper

Roadmap position:
- This is a developer-workflow improvement alongside the active Sprint 1 UI work.

Completed in this slice:
- Added scripts/local-launch.ps1 to stop the existing Compose stack, rebuild the API image, recreate services, and wait for /health and /app to return 200.
- Added optional flags for a fresh Postgres volume refresh and log following.
- Documented the helper in README.md as the recommended local rebuild/relaunch path for UI and user-testing updates.
- Verified the helper by running it successfully against the local Compose stack.

Validation:
- powershell -ExecutionPolicy Bypass -File .\\scripts\\local-launch.ps1

Next UI/dev workflow steps:
- Use local-launch.ps1 as the standard refresh path when checking UI changes in Docker Compose.
## 2026-04-09 - Sprint 1 UI Slice: Standardize Workspace Shells

Roadmap position:
- Sprint 1 remains active.
- This checkpoint standardized the workspace layer under the already-shared route shells.

Completed in this slice:
- Added shared workspace-shell helpers for queue cards and path sections.
- Moved Organization, Forms, Responses, Administration, Reports, and Dashboards workspace shells onto the same queue/path/workspace rendering pattern.
- Preserved the existing supported actions, route bridges, and underlying screens while removing more duplicated route-console markup.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 blocked because Docker Desktop / the Docker daemon was unavailable on the machine at run time.

Next UI steps:
- Continue Sprint 1 route-level browse/detail framing.
- Keep tightening the product-area experience without adding unsupported UI behavior.
## 2026-04-09 - Sprint 1 UI Slice: Keep Product-Area Anchors Out Of Builder Language

Roadmap position:
- Sprint 1 remains active.
- This checkpoint continues tightening route-level framing while staying inside the existing backend-supported UI surfaces.

Completed in this slice:
- Continued standardizing workspace-level route shells through shared queue/path helpers.
- Reduced route-specific UI drift under the product-area shells.
- Preserved existing actions and route bridges while moving the route consoles closer to one consistent application-shell pattern.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 was blocked because the Docker daemon was unavailable at run time.

Next UI steps:
- Continue Sprint 1 route-level browse/detail framing.
- Keep reducing builder-era language where product routes still inherit it from reused screens.
## 2026-04-09 - Sprint 1 UI Slice: Replace Builder-Era Route Anchors

Roadmap position:
- Sprint 1 remains active.
- This checkpoint keeps aligning route-level UI language with the product-area shell without changing supported behavior.

Completed in this slice:
- Replaced builder-era route anchors like hierarchy-admin-screen, form-admin-screen, report-admin-screen, and submission-screen with route-appropriate screen IDs.
- Updated product and internal route links so route navigation no longer advertises admin-first anchor names where the screens are reused.
- Kept the underlying actions and reused screens intact.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 browse/detail framing.
- Keep reducing inherited builder-era language on shared screens and route entry points.
## 2026-04-09 - Sprint 1 UI Slice: Remove More Builder-Era Screen Language

Roadmap position:
- Sprint 1 remains active.
- This checkpoint continues the route-language cleanup inside reused screens.

Completed in this slice:
- Renamed reused screen headers and labels to better match product-area and administration route language.
- Responses screens now use Response Entry, Response Review, and Response Reports language.
- Administration screens now use Organization Setup, Forms Configuration, and Reporting Configuration language.
- Migration fixture screen now uses Fixture Intake and Validation language.
- Updated route tests and smoke expectations to match the new screen wording.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 was blocked because the Docker daemon was unavailable at run time.

Next UI steps:
- Continue Sprint 1 browse/detail framing on the product routes.
- Keep reducing the remaining internal-builder feel on reused screens without adding unsupported UI behavior.
## 2026-04-09 - Sprint 1 UI Slice: Make Reused Screens Route-Aware

Roadmap position:
- Sprint 1 remains active.
- This checkpoint improves reused screen framing on the product routes without changing underlying behavior.

Completed in this slice:
- Made the shared organization/forms management screens route-aware so Organization and Forms routes no longer inherit administration-first titles.
- Organization route now renders product-surface labels like Organization Screen and Organization Directory.
- Forms route now renders product-surface labels like Forms Screen and Forms Directory.
- Administration still keeps Organization Setup and Forms Configuration language.
- Added test coverage to prove the product routes and administration route now diverge correctly in visible labeling.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 browse/detail framing.
- Keep reducing the remaining internal-builder feel on reused screens and route entry points.
## 2026-04-09 - Sprint 1 checkpoint: route interaction language cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Updated product-route cards and task buttons to emphasize browse, review, and view actions instead of generic builder-style `Open` and `Choose` wording.
  - Kept explicit create/configure language scoped to Administration.
  - Tightened reporting and response route wording so the visible UI reads more like a product surface and less like a reused internal workbench.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
  - `.\scripts\smoke.ps1` was blocked because the Docker daemon was unavailable (`dockerDesktopLinuxEngine` pipe not found).
- Next step:
  - Continue Sprint 1 by improving browse/detail framing on the product routes while keeping UI work bounded to the backend-supported workflows already implemented.

## 2026-04-09 - Sprint 1 checkpoint: product workspace framing

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Renamed product-route page shells from `Console` wording to `Workspace` wording.
  - Renamed route-side rail sections from `Queues` to `Browse` on product surfaces.
  - Renamed route-side step sections from generic `Path` wording to `Flow` wording on product surfaces.
  - Kept Administration as `Configuration Console` with `Management Queues`, since it remains the internal management surface.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
  - `.\scripts\smoke.ps1` was blocked because the Docker daemon was unavailable (`dockerDesktopLinuxEngine` pipe not found).
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing inside the product workspaces while leaving creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: product screen framing cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Updated product-route inner screen framing so reused screens read less like technical shells and more like application surfaces.
  - Replaced product-route subheadings such as `Organization Screen`, `Forms Screen`, and `Reports Screen` with `Workspace` wording.
  - Replaced product-route context labels such as `Current ... Context` with selection-oriented labels where those routes act as browse/detail surfaces.
  - Kept Administration-specific configuration wording unchanged.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing and output presentation on product routes while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: route-specific detail panels

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Replaced the shared `Screen Output` panel on app routes with route-specific detail/result panels.
  - Replaced the shared `Raw Output` panel with `Raw API Activity` on product routes and `Raw API Output` on internal routes.
  - Made the bottom-of-page framing read like route-specific detail/result space instead of a leftover workbench/debug panel.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing on the remaining product-route internals while keeping builder and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: landing label consistency

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Normalized the remaining mixed landing-card labels on product routes.
  - Home cards now use `Go to ...` wording where they navigate to another product area.
  - Reporting landing cards now use browse/review/view wording instead of `Open ... Workspace`.
  - Dashboard landing cards now use `View Demo Preview` rather than another generic `Open` label.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing remaining low-level/product-route friction while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: shared sidebar wording cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Renamed the shared selection panel from `Selection Context` to `Current Selections`.
  - Reframed the shared session/summary actions with more application-oriented wording:
    - `Sign In`
    - `Session Status`
    - `Sign Out`
    - `Refresh Summary`
  - Updated route tests and smoke expectations to match the new shared-shell labels.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing remaining generic shell/test-harness wording on product routes while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: product-route description cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Removed another layer of transitional `bridge`, `catch-up`, and similar wording from product-route descriptions.
  - Reframed the home, organization, forms, responses, reports, and dashboards descriptions so they read more like stable application areas.
  - Corrected a duplicated forms landing description field introduced during the copy pass.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing the remaining generic product-route wording and tightening the visible information architecture before moving on to the next sprint.

## 2026-04-09 - Sprint 1 checkpoint: dashboard action-row consistency

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Normalized the dashboard route action row to match the rest of the shared shell.
  - Updated dashboard route action labels to:
    - `Sign In`
    - `Session Status`
    - `Sign Out`
    - `Refresh Summary`
  - Added dashboard-route assertions so the shared-shell wording stays consistent across product areas.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing the remaining generic product-route wording and tightening the visible information architecture before moving on to the next sprint.

## 2026-04-09 - Sprint 1 checkpoint: product header action cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Removed demo/setup shortcuts from product-area header action rows.
  - Product routes now use refresh-oriented header actions instead:
    - `Refresh Organization`
    - `Refresh Forms`
    - `Refresh Responses`
    - `Refresh Reports`
    - `Refresh Dashboards`
  - Home and internal areas remain the place for demo seeding and setup-oriented entry points.
  - Updated route tests and smoke expectations to match the new product-header contract.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening the remaining visible information architecture and deciding whether the shared-shell/navigation sprint is complete enough to move to the next sprint.

## 2026-04-10 - Sprint 1 complete

- Roadmap position:
  - Sprint 1, Shared Shell and Product-Area Navigation, is complete.
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is the next planned sprint.
- Completion rationale:
  - split product-area navigation is in place
  - shared shell framing is consistent across product and internal areas
  - product routes now read as workspaces and viewing destinations rather than mixed builder consoles
  - Administration and Migration remain visible but clearly internal/operator scoped
- Current review point:
  - the UI is now in a good place to review Sprint 1 shell/navigation progress before moving deeper into home-surface and role-ready work

## 2026-04-10 - Sprint 2 start: home surface and layout correction

- Roadmap position:
  - Sprint 1 is closed.
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is now active.
- Scope:
  - start the shared-home refactor without adding backend scope
  - fix the shared `task-panel context-panel` layout overflow
- Completed:
  - corrected the shared task/context grid so `section.task-panel.context-panel` breaks below the preceding panel instead of escaping the parent layout
  - removed demo seeding from the shared home header actions
  - refactored `/app` into clearer home modules:
    - product areas
    - current deployment readiness
    - current workflow context
    - internal areas
  - wired the home readiness module to the existing `/api/app/summary` surface
  - wired the home current-context module to the existing shared selection state
  - moved demo setup emphasis into Administration through a local testing utility card
  - updated the roadmap to remove stale Sprint 1 carry-forward notes and mark Sprint 2 active
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - continue Sprint 2 by tightening the shared home modules and reducing any remaining transitional copy on product-facing routes

## 2026-04-10 - Sprint 2 checkpoint: remove visible ID entry fields

- Roadmap position:
  - Sprint 2 remains active.
  - Scope stays bounded to backend-supported UI catch-up work.
- Completed:
  - removed visible `ID` entry fields from the rendered application screens
  - kept selection-driven state in hidden inputs so existing controller flows still work
  - updated task/context panels so creation flows no longer imply that users should type or override database-assigned identifiers
  - added route-test coverage to prevent the main product and administration screens from regressing back to visible `ID` fields
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - continue Sprint 2 by tightening the shared home modules and reducing remaining transitional copy on product-facing routes

## 2026-04-10 - Sprint 2 complete

- Roadmap position:
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is complete.
  - Sprint 3, Organization And Forms Product Surfaces, is the next planned sprint.
- Completed:
  - added explicit role-ready home modules to the shared home without introducing separate role routes
  - removed remaining transitional product-facing copy on the shared home and key reports/dashboards surfaces
  - removed the demo dashboard shortcut from the Dashboards product surface
  - completed the Sprint 2 contract:
    - shared home as the real product entry point
    - structural role readiness only
    - existing summary and selection surfaces reused
    - demo/testing utilities scoped to internal placement
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - begin Sprint 3 on Organization and Forms product surfaces

## 2026-04-10 - Sprint 2.5 complete

- Roadmap position:
  - Sprint 2.5, Entity CRUD/List Surfaces, is complete.
  - Sprint 3, Organization And Forms Product Surfaces, is next.
- Completed:
  - inserted Sprint 2.5 into the roadmap between Sprint 2 and Sprint 3
  - added runtime organization detail retrieval with `GET /api/nodes/{node_id}`
  - converted product routes to explicit entity list/detail surfaces for:
    - Organization
    - Forms
    - Responses
    - Reports
    - Dashboards
  - split list output from selected-detail output in the browser controllers
  - added clearer Administration create/edit entry points for top-level:
    - Organization
    - Form
    - Report
    - Dashboard
  - kept response creation/editing in Responses through the existing draft lifecycle
  - removed more product-facing ID-driven friction by using selection-driven detail flows and hiding explicit ID entry from the rendered screens
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\hierarchy.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\tests\demo_flow.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
  - `.\scripts\local-launch.ps1` passed
- Next step:
  - resume Sprint 3 on deeper Organization and Forms product surfaces

## 2026-04-10 - Sprint 2.5 replacement complete

- Roadmap position:
  - Sprint 2.5 remains complete, but it now reflects the dedicated-screen replacement rather than the earlier workspace-panel interpretation.
  - Sprint 3, Organization And Forms Product Surfaces, remains next.
- Completed:
  - replaced the product-area workspace-panel approach with dedicated navigable screens for:
    - Organization
    - Forms
    - Responses
    - Reports
    - Dashboards
  - added explicit product-area routes for:
    - list
    - create
    - detail
    - edit
  - kept IDs in route paths only and removed them from visible form fields
  - moved top-level entity CRUD/view workflows into product areas and stopped extending the internal admin/testing screens for those flows
  - kept Administration as an internal advanced/configuration landing area with links to legacy tooling
  - kept Responses as the canonical draft/edit/review surface:
    - dedicated start screen
    - dedicated detail screen
    - dedicated draft-only edit screen
    - submitted responses remain read-only
  - kept Report create/edit on dedicated pages with a minimal binding editor required by the current backend
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - begin Sprint 3 by deepening Organization and Forms product surfaces now that the dedicated top-level entity screens are in place

## 2026-04-10 - Sprint 3 sequencing update

- Decision:
  - Sprint 3 should begin with a frontend code organization pass.
- Added to the roadmap:
  - the first Sprint 3 task is now to refactor the product UI code by route/screen before adding deeper Organization and Forms behavior
- Reason:
  - the UI is now route-driven
  - keeping the current large page/controller files in place would slow Sprint 3 work and increase regression risk
- Immediate next step:
  - split the shared shell/navigation code from Organization and Forms screen modules, then continue Sprint 3 feature work on top of that structure

## 2026-04-10 - UAT demo seed dataset and local launch integration

- Completed:
  - expanded the existing deterministic demo seed in `D:\Projects\dms-migration\tessara\crates\tessara-api\src\demo.rs`
  - integrated automatic demo seeding into `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1`
  - added `D:\Projects\dms-migration\tessara\scripts\seed-demo-data.ps1` for manual reseeding against a running Compose stack
- Seed shape:
  - `Partner -> Program -> Activity -> Session`
  - `2` partners, `4` programs, `6` activities, `8` sessions
  - metadata coverage across all supported field types: `text`, `number`, `boolean`, `date`, `single_choice`, `multi_choice`
  - one published form family per hierarchy level
  - `2` submitted responses and `1` draft response per form family
  - `4` reports and `1` compact dashboard for UAT navigation and review
- Launch behavior:
  - normal `.\scripts\local-launch.ps1` now preserves existing local data and ensures the demo dataset exists
  - `.\scripts\local-launch.ps1 -FreshData` still rebuilds from a clean database volume, then reseeds the demo dataset
  - no build-time or container-startup auto-seeding was added
- Supporting changes:
  - updated `D:\Projects\dms-migration\tessara\README.md` to document the new seeding flow
  - updated `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` and `D:\Projects\dms-migration\tessara\crates\tessara-api\tests\demo_flow.rs` for the richer seeded dataset
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
  - `.\scripts\local-launch.ps1` passed and left the refreshed stack running

## 2026-04-10 - Organization List Loader Fix
- Fixed a JavaScript syntax error in 	essara-web/src/app_script.rs that prevented product-route loaders from running.
- Rebuilt and relaunched the local Docker stack with scripts/local-launch.ps1 and verified seeded organization data renders again on /app/organization.

## 2026-04-10 - Role-Based Screen Access

- Implemented role-aware access scaffolding across the dedicated application screens:
  - `admin/system`
  - `scoped operator/partner`
  - `respondent/client/parent`
- Backend changes in `D:\Projects\dms-migration\tessara\crates\tessara-api`:
  - added migration `011_role_access.sql` for:
    - account credentials
    - account-to-node scope assignments
    - parent/subordinate respondent relationships
  - extended auth context to expose:
    - `role_family`
    - assigned scope nodes
    - subordinate respondents
  - added recursive effective-scope resolution for operator access
  - split read access from write access for product APIs:
    - hierarchy reads scoped for operators
    - forms readable through `/api/forms`
    - reports and dashboards filtered for operators
    - responses filtered by scoped nodes or respondent context
  - added `/api/responses/options` for role-aware response-start choices
  - made `/api/app/summary` authenticated and role-aware instead of report-admin-only
- Frontend changes in `D:\Projects\dms-migration\tessara\crates\tessara-web`:
  - added `/app/login`
  - removed automatic admin login from the product shell
  - added role-aware navigation hiding and direct-route access guards
  - kept create/edit product screens admin-only for Organization, Forms, Reports, and Dashboards
  - kept Responses as the create/update surface for operators and respondents
  - added respondent context switching in Responses for parent/subordinate flows
- Demo/UAT support:
  - expanded the demo seed to create:
    - operator account
    - parent account
    - respondent account
    - child respondent account
  - assigned operator scope to non-root hierarchy nodes so descendant scoping is exercised
  - seeded subordinate respondent relationships and pending assigned response starts
  - updated local launch output to show all demo credentials
- Validation:
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` passed
  - `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` passed

## 2026-04-10 - Roadmap alignment update

- Updated `D:\Projects\dms-migration\tessara\docs\roadmap.md` so it reflects the current implemented state instead of the earlier UI-only snapshot.
- Corrected the roadmap to include already-landed baseline work for:
  - explicit login
  - role-aware navigation and route guards
  - scoped operator access with descendant expansion
  - parent/subordinate respondent context
  - UAT demo seeding through `local-launch.ps1`
- Updated current position so:
  - Sprint 3 is marked active
  - the next concrete milestone is the route/screen-oriented frontend refactor
- Replaced the stale `Immediate Next Sprint` section that still incorrectly said to start with Sprint 1.

## 2026-04-13 - Access Administration And Delegation Closeout

- Closed the access/admin sprint on `D:\Projects\dms-migration\tessara` with the first application-grade admin/auth vertical slice.
- Backend changes:
  - replaced hard-coded `role_family` responses with capability and scope metadata
  - extended `/api/me` and user detail responses to include roles, capabilities, scope nodes, and delegations
  - replaced subordinate-respondent storage and API usage with generic `account_delegations`
  - added `GET /api/admin/users/{account_id}/access`
  - added `POST /api/admin/roles` so admins can create new role bundles, not just edit seeded roles
  - updated response-context access to use delegation resolution through `delegate_account_id`
- Frontend changes:
  - completed inline login failure handling without dropping users into generic error output
  - added current-user summary content in the shell/home
  - upgraded role edit/create to a filterable capability grid and added the dedicated `/app/administration/roles/new` route
  - upgraded user access to a filterable scope/delegation management surface with effective-access summary
  - generalized delegated response context so it is account-based, not tied to a hard-coded respondent-family assumption
- Demo/UAT changes:
  - renamed demo delegation accounts to `delegator@tessara.local` and `delegate@tessara.local`
  - updated `D:\Projects\dms-migration\tessara\scripts\seed-demo-data.ps1`, `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`, and `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` to the delegation model
- Roadmap update:
- updated `D:\Projects\dms-migration\tessara\docs\roadmap.md`
  - marked Sprint 1A and Sprint 1B complete
  - left Sprint 1C as the next organization-focused slice
- Validation:
  - `cargo fmt --all` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` passed
  - `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` passed and left the refreshed stack running
## 2026-05-18 - Workflow-Mediated Response Start Cleanup

- Confirmed response access now derives from workflow assignment ownership or delegation for respondent-style access:
  - submission access loads through `submissions.workflow_assignment_id`
  - response starts use workflow assignment start authorization
  - delegated starts/options resolve through account delegation before listing pending assignment work
- Clarified that admin/operator response access remains capability/scope-gated, but no response path relies on direct form assignment storage.
- Removed the separate form/node response-start concept from the current implementation notes:
  - `/api/responses/options` is assignment-only
  - form-first "Assign Form" is a UI convenience over generated single-form workflows plus normal workflow assignments
  - response drafts start through `/api/workflow-assignments/{workflow_assignment_id}/start`
- Tightened DTO naming around assignment-backed response start options so future work does not reintroduce a manual form/node start mode.

## 2026-05-19 - Form Scope Direction

- Captured the product direction that forms should not be intrinsically scoped to a node type.
- Captured the related product direction that workflows should be available at an explicit list of concrete nodes, not constrained by a workflow node type.
- Workflow steps should become the owner of context and target semantics, including future nonlinear branching, forward-passed form data, prefills, hidden or locked carried-forward values, and derived target nodes.
- Started removing form scope as a workflow compatibility rule:
  - workflow step form-version options are no longer filtered by form node scope
  - workflow assignment candidates no longer require every step form scope to appear under the assignment node
  - workflow response starts no longer retarget a step to a descendant node solely because the form had a legacy scope node type
- Added `workflow_available_nodes` so assignment candidates are driven by explicit workflow/node availability.
- Left persisted form `scope_node_type_id` and workflow `workflow_node_type_id` in place temporarily because generated single-form workflow creation still needs an explicit replacement for choosing default workflow availability.

## 2026-07-22 - Sprint 6B1 container foundation implementation

- Implemented the first independently deployed module vertical slice on `codex/sprint-6b1`:
  - versioned deterministic deployment/plan/receipt/rollback contracts;
  - `tessara-deploy` validate, plan, apply, status, and rollback commands;
  - one PostgreSQL cluster with isolated Core, deployment-control, and Scoped Records databases and distinct roles;
  - Traefik same-origin routing with default-deny discovery and a Core-rendered unavailable fallback;
  - a non-production Scoped Records full-stack module with migrations, SSR routes, CRUD API, probes, diagnostics, and retained data;
  - persisted Module Release, Module Instance, and deployment receipt history readback;
  - the approved additive Screen A, B, and C Module Management changes, including database component readback and direct receipt navigation.
- Regression and local acceptance evidence:
  - contract/deploy tests passed;
  - reference-module and web tests passed;
  - API/web checks and formatting passed;
  - existing Module Management Playwright suite passed all 5 scenarios;
  - contract acceptance and live install/upgrade/rollback/outage/recovery/database-isolation acceptance passed;
  - legacy smoke and UAT passed against the canonical application stack.
- The disposable canonical smoke stack and its disposable test volume were removed after browser verification. The Sprint 6B1 stack remains available at `http://127.0.0.1:8180` for review.
- Product direction changed after implementation review: Sprint 6B1 uses a curated Tessara development release and does not require operators to install Cosign. Cryptographic admission, catalog distribution, and production lifecycle management of verified third-party containers are future platform work.
- Removed recorded publisher-verification input from the current `tessara-deploy apply` workflow and changed UI readback from `verified` to explicit `curated release` provenance so the product does not overstate its trust evidence.
- Rebuilt the Sprint 6B1 images, deleted the development PostgreSQL volume, recreated the cluster from the then-current pre-closeout migrations, and reran live install/upgrade/rollback/outage/recovery acceptance against the fresh schema. The closeout candidate subsequently squashes that schema into one baseline and replaces this intermediate evidence.
- Closeout remains gated only on persisting the complete curated `ModuleManifestV1` for real releases and projecting it through descriptor download plus the preserved Declarations, Contracts, Capabilities, Dependencies, Resources, and Navigation tabs. The current sanitized read model proves lifecycle/deployment behavior but its explanatory tab placeholders are not final accepted Screen B behavior.
